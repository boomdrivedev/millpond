//! The per-host state: each adapter's cursor and last run's status, one
//! file per adapter under the inbox's state folder, written whole by
//! replacing it, and a lock beside it so an adapter never runs twice at
//! once (22, 26, 86 to 88; C.3). Losing it loses no record of the inbox:
//! the next run starts with no cursor (25).

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

/// One adapter's state on this host.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AdapterState {
    /// The cursor its last complete run kept (23).
    pub cursor: Option<String>,
    pub status: Option<Status>,
}

/// What an adapter's last run on this host did (26).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub ran_at: String,
    pub complete: bool,
    /// Why it was not run, where it was not (21).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_run: Option<String>,
    /// The run's `weir-report/1`: what it took, held and refused, and each
    /// item the adapter named as failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<serde_json::Value>,
    /// The last of what the adapter reported when it failed, each secret's
    /// value taken out (75).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_report: Option<String>,
}

fn file(dir: &Path, adapter: &str) -> PathBuf {
    dir.join(format!("{adapter}.json"))
}

/// The adapter's state, empty where none is kept or it cannot be read.
pub fn read(dir: &Path, adapter: &str) -> AdapterState {
    fs::read_to_string(file(dir, adapter))
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

/// Writes the adapter's state whole, by replacing its file.
pub fn write(dir: &Path, adapter: &str, state: &AdapterState) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let path = file(dir, adapter);
    let partial = path.with_extension("json.partial");
    let text = serde_json::to_string_pretty(state).map_err(|error| error.to_string())?;
    fs::write(&partial, text).map_err(|error| format!("{}: {error}", partial.display()))?;
    fs::rename(&partial, &path).map_err(|error| format!("{}: {error}", path.display()))
}

/// A lock held while it lives.
pub struct Lock {
    path: PathBuf,
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Takes the lock `name` in `dir`, naming its holder, breaking one older
/// than `stale_after`; refused while another holds it.
pub fn lock(dir: &Path, name: &str, holder: &str, stale_after: Duration) -> Result<Lock, String> {
    fs::create_dir_all(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let path = dir.join(format!("{name}.lock"));
    for _ in 0..2 {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                let _ = writeln!(file, "{holder}");
                return Ok(Lock { path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let age = fs::metadata(&path)
                    .and_then(|meta| meta.modified())
                    .ok()
                    .and_then(|at| SystemTime::now().duration_since(at).ok())
                    .unwrap_or_default();
                if age <= stale_after {
                    let holder = fs::read_to_string(&path).unwrap_or_default();
                    return Err(format!(
                        "{name} is running: {} holds its lock",
                        holder.trim()
                    ));
                }
                let _ = fs::remove_file(&path);
            }
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
    }
    Err(format!("{name}'s lock could not be taken"))
}
