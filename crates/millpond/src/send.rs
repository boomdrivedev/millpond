//! A send to a place, by flume (89 to 92; C.5, C.6), and the merge queue's
//! record of the place's branch, by sluice (93, 94; C.7).
//!
//! The launch is `flume launch` with the rule's kind in the rule's
//! repository, the place named `<item>--<n>`, the item's brief by
//! `--brief-file`, its parameters by `--params-file`, the priority the
//! settings give the rule's level, and a mark naming the inbox, the item,
//! the send's number and the rule that asked (51, 52, 131). The program is
//! `flume` on PATH, or the one `MILLPOND_FLUME` names; the record is read
//! from `sluice status <base> --all --json` in the repository, or the
//! program `MILLPOND_SLUICE` names.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

/// The variable naming the program run in place of `flume`.
pub const FLUME: &str = "MILLPOND_FLUME";

/// The variable naming the program run in place of `sluice`.
pub const SLUICE: &str = "MILLPOND_SLUICE";

/// One launch of a place for an item's send.
#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    pub kind: String,
    /// `<item>--<n>`.
    pub name: String,
    /// The repository as the rule names it.
    pub repository: String,
    pub brief: String,
    pub params: Value,
    pub priority: i64,
    pub mark: String,
}

impl Launch {
    /// The send's target as the item's record keeps it: the place, by
    /// kind and name, in the repository (51).
    pub fn target(&self) -> String {
        target(&self.kind, &self.name, &self.repository)
    }

    /// The arguments of `flume launch`, the brief and the parameters in
    /// the two files named.
    pub fn args(&self, brief: &Path, params: &Path) -> Vec<String> {
        vec![
            "launch".into(),
            "--json".into(),
            "--kind".into(),
            self.kind.clone(),
            "--name".into(),
            self.name.clone(),
            "--repository".into(),
            expand(&self.repository).to_string_lossy().into_owned(),
            "--brief-file".into(),
            brief.to_string_lossy().into_owned(),
            "--params-file".into(),
            params.to_string_lossy().into_owned(),
            "--priority".into(),
            self.priority.to_string(),
            "--mark".into(),
            self.mark.clone(),
        ]
    }

    /// Runs the launch: what flume printed, or why it refused.
    pub fn run(&self) -> Result<String, String> {
        let dir = tempfile::tempdir().map_err(|error| error.to_string())?;
        let brief = dir.path().join("brief.md");
        let params = dir.path().join("params.json");
        fs::write(&brief, &self.brief).map_err(|error| error.to_string())?;
        let text = serde_json::to_string_pretty(&self.params).map_err(|e| e.to_string())?;
        fs::write(&params, text + "\n").map_err(|error| error.to_string())?;
        let program = std::env::var(FLUME).unwrap_or_else(|_| "flume".into());
        let output = Command::new(&program)
            .args(self.args(&brief, &params))
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("{program} could not be run: {error}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(format!(
                "flume refused the launch of {}: {}",
                self.name,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
        }
    }
}

/// A place's target: `place:<kind>:<name>:<repository>`.
pub fn target(kind: &str, name: &str, repository: &str) -> String {
    format!("place:{kind}:{name}:{repository}")
}

/// The kind, the place's name and the repository a place's target names.
pub fn place_of(target: &str) -> Option<(&str, &str, &str)> {
    let rest = target.strip_prefix("place:")?;
    let mut parts = rest.splitn(3, ':');
    Some((parts.next()?, parts.next()?, parts.next()?))
}

/// A repository as a rule names it, `~/` read as the home folder.
pub fn expand(repository: &str) -> PathBuf {
    match repository.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(repository),
    }
}

/// The branch the place `name` landed onto and the landing's revision, as
/// sluice recorded it, or none while it has not landed (55, 93, 94). The
/// place's branch is `<prefix>/<name>`, its kind's prefix; its submission
/// is the one from that branch.
pub fn landed(repository: &str, name: &str) -> Result<Option<(String, String)>, String> {
    let dir = expand(repository);
    let base = default_branch(&dir);
    let program = std::env::var(SLUICE).unwrap_or_else(|_| "sluice".into());
    let output = Command::new(&program)
        .args(["status", &base, "--all", "--json"])
        .current_dir(&dir)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("{program} could not be run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "sluice could not read {base}'s record in {}: {}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let record: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("sluice's record is not JSON: {error}"))?;
    let suffix = format!("/{name}");
    let found = record["submissions"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|submission| {
            submission["state"] == "landed"
                && submission["from"]["head"]
                    .as_str()
                    .is_some_and(|head| head.ends_with(&suffix))
        });
    Ok(found.and_then(|submission| {
        Some((
            submission["branch"].as_str()?.to_string(),
            submission["landed_as"].as_str()?.to_string(),
        ))
    }))
}

/// The repository's default branch: what `origin/HEAD` names, else `main`.
fn default_branch(dir: &Path) -> String {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .strip_prefix("origin/")
                .map(str::to_string)
        })
        .unwrap_or_else(|| "main".into())
}
