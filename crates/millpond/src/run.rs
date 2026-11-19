//! A run: one start of one adapter by millpond until it exits (20 to 26,
//! 30, 78; C.1). It is handed the configuration the settings give it,
//! checked against its declaration's schema, the secrets its declaration
//! names from this host's folder and no others, an empty raw place of its
//! own and the cursor of its last complete run; its lines are taken, and
//! its cursor and status kept on this host.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use weir_capture::declaration::CAPTURE_CONFIG;
use weir_capture::{check_config, read_declarations, CameIn, Declaration, Report};

use crate::clock;
use crate::host::HostConfig;
use crate::state::{self, AdapterState, Status};
use crate::store;
use crate::take::Taker;

/// How many lines of what a failed adapter reported its status keeps.
const TAIL: usize = 20;

/// A declaration and the folder its schema lies in.
pub struct Declared {
    pub declaration: Declaration,
    pub beside: PathBuf,
}

/// The one adapter a declaration file declares, or the one of the name
/// given among those it declares.
pub fn declared(path: &Path, name: Option<&str>) -> Result<Option<Declared>, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let declarations =
        read_declarations(&text).map_err(|error| format!("{}: {error}", path.display()))?;
    let beside = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let found = match name {
        Some(name) => declarations.into_iter().find(|d| d.name == name),
        None => match <[Declaration; 1]>::try_from(declarations) {
            Ok([only]) => Some(only),
            Err(declarations) => {
                return Err(format!(
                    "{} declares {} adapters, and a run names one",
                    path.display(),
                    declarations.len()
                ))
            }
        },
    };
    Ok(found.map(|declaration| Declared {
        declaration,
        beside,
    }))
}

/// The declaration of the adapter `name` among the host's (C.1 describe).
pub fn find(host: &HostConfig, name: &str) -> Result<Declared, String> {
    for path in &host.declarations {
        if let Some(found) = declared(path, Some(name))? {
            return Ok(found);
        }
    }
    Err(format!(
        "no declaration this host lists declares the adapter `{name}`"
    ))
}

/// Runs the adapter as its settings say (20, 21), on this host only (3).
pub fn configured(taker: &Taker, host: &HostConfig, name: &str) -> Result<Report, String> {
    let (_, _, settings) = store::read(&taker.inbox.branch)?;
    let setting = settings
        .adapters
        .get(name)
        .ok_or_else(|| format!("the inbox's settings name no adapter `{name}`"))?;
    if setting.host != taker.machine {
        return Err(format!(
            "`{name}` runs on {}, and this host is {}",
            setting.host, taker.machine
        ));
    }
    let declared = find(host, name)?;
    run(
        taker,
        &declared,
        &setting.config,
        host.secrets.as_deref(),
        settings.lock_stale,
    )
}

/// One run of the declared adapter: refused, with its status saying why,
/// when its configuration does not fit its schema or a secret it names is
/// not held here (21); otherwise started with a fresh raw place, its lines
/// taken, its cursor moved only by a complete run (23, 24).
pub fn run(
    taker: &Taker,
    declared: &Declared,
    config: &serde_json::Value,
    secrets: Option<&Path>,
    stale_after: std::time::Duration,
) -> Result<Report, String> {
    let name = &declared.declaration.name;
    let dir = taker.inbox.state_dir();
    let _lock = state::lock(&dir, name, taker.machine, stale_after)?;
    let mut kept = state::read(&dir, name);
    let ran_at = clock::stamp();
    let not_run = |kept: &mut AdapterState, why: String| -> String {
        kept.status = Some(Status {
            ran_at: ran_at.clone(),
            complete: false,
            not_run: Some(why.clone()),
            report: None,
            last_report: None,
        });
        let _ = state::write(&dir, name, kept);
        format!("`{name}` was not run: {why}")
    };
    let schema_path = declared.beside.join(&declared.declaration.config);
    let schema = fs::read_to_string(&schema_path)
        .map_err(|error| format!("{}: {error}", schema_path.display()))
        .and_then(|text| serde_json::from_str(&text).map_err(|error| error.to_string()));
    let schema: serde_json::Value = match schema {
        Ok(schema) => schema,
        Err(why) => return Err(not_run(&mut kept, format!("its schema: {why}"))),
    };
    if let Err(error) = check_config(&schema, config) {
        return Err(not_run(&mut kept, error.to_string()));
    }
    let mut values = Vec::new();
    for secret in &declared.declaration.secrets {
        let value = secrets
            .map(|dir| dir.join(secret))
            .and_then(|path| fs::read_to_string(path).ok());
        match value {
            Some(value) => values.push((secret.clone(), value.trim_end().to_string())),
            None => {
                return Err(not_run(
                    &mut kept,
                    format!("this host holds no secret `{secret}`"),
                ))
            }
        }
    }
    let raw = tempfile::tempdir().map_err(|error| error.to_string())?;
    let config_dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let config_file = config_dir.path().join("config.json");
    fs::write(&config_file, config.to_string()).map_err(|error| error.to_string())?;
    let invocation = declared
        .declaration
        .invocation(raw.path(), kept.cursor.as_deref());
    let mut command = Command::new(&invocation.args[0]);
    command
        .args(&invocation.args[1..])
        .current_dir(&declared.beside)
        .env(CAPTURE_CONFIG, &config_file)
        .stdin(Stdio::null());
    for (secret, value) in &values {
        command.env(secret, value);
    }
    let output = command
        .output()
        .map_err(|error| not_run(&mut kept, format!("it could not be started: {error}")))?;
    let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    let report = taker
        .take_lines(&lines, Some(raw.path()), CameIn::Run(name.clone()))?
        .finish(output.status.success());
    let last_report = (!report.complete).then(|| {
        let said = String::from_utf8_lossy(&output.stderr);
        let lines: Vec<&str> = said.lines().collect();
        let tail = lines[lines.len().saturating_sub(TAIL)..].join("\n");
        values
            .iter()
            .filter(|(_, value)| !value.is_empty())
            .fold(tail, |tail, (secret, value)| {
                tail.replace(value, &format!("[{secret}]"))
            })
    });
    if report.complete {
        if let Some(cursor) = &report.cursor {
            kept.cursor = Some(cursor.clone());
        }
    }
    kept.status = Some(Status {
        ran_at,
        complete: report.complete,
        not_run: None,
        report: serde_json::from_str(&report.to_json()).ok(),
        last_report,
    });
    state::write(&dir, name, &kept)?;
    Ok(report)
}
