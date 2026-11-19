//! The host's configuration (C.3): the inboxes this host keeps, the
//! adapters' declarations it can run, where its secrets lie, and its
//! directories under XDG's.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;
use weir_branch::Branch;
use weir_capture::content_hash;

/// The variable naming the host's configuration file in place of
/// `$XDG_CONFIG_HOME/millpond/config.toml`.
pub const CONFIG: &str = "MILLPOND_CONFIG";

/// The host's configuration file, as written.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    /// The machine's name in pointers and settings; its short hostname
    /// where none is given.
    pub host: Option<String>,
    /// The folder holding one file per secret, named as declared.
    pub secrets: Option<PathBuf>,
    /// The adapters' declaration files this host can run (C.1 describe).
    #[serde(default)]
    pub declarations: Vec<PathBuf>,
    #[serde(default)]
    pub inbox: Vec<InboxEntry>,
}

/// One inbox this host keeps: its name, its repository, and its branch.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InboxEntry {
    pub name: String,
    /// A local repository's path, or a remote's address.
    pub repository: String,
    #[serde(default = "default_branch")]
    pub branch: String,
}

fn default_branch() -> String {
    "inbox".to_string()
}

/// One XDG base directory, or its default under `$HOME`.
pub fn xdg(variable: &str, fallback: &str) -> PathBuf {
    std::env::var_os(variable)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(fallback)
        })
}

impl HostConfig {
    /// The host's configuration, or none where the file is missing.
    pub fn load() -> Result<HostConfig, String> {
        let path = std::env::var_os(CONFIG)
            .map(PathBuf::from)
            .unwrap_or_else(|| xdg("XDG_CONFIG_HOME", ".config").join("millpond/config.toml"));
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(HostConfig::default()),
            Err(error) => Err(format!("{}: {error}", path.display())),
        }
    }

    /// This machine's name: the configuration's, or its short hostname.
    pub fn machine(&self) -> Result<String, String> {
        if let Some(host) = &self.host {
            return Ok(host.clone());
        }
        let output = Command::new("hostname")
            .stdin(Stdio::null())
            .output()
            .map_err(|error| format!("hostname could not be run: {error}"))?;
        let name = String::from_utf8_lossy(&output.stdout);
        let short = name.trim().split('.').next().unwrap_or_default();
        if short.is_empty() {
            return Err("hostname named no machine".into());
        }
        Ok(short.to_string())
    }

    /// The inbox named `name`, or the only one where none is named.
    pub fn inbox(&self, name: Option<&str>) -> Result<&InboxEntry, String> {
        match name {
            Some(name) => self
                .inbox
                .iter()
                .find(|entry| entry.name == name)
                .ok_or_else(|| format!("this host keeps no inbox named `{name}`")),
            None => match self.inbox.as_slice() {
                [only] => Ok(only),
                [] => Err("this host keeps no inbox: name one with --inbox <repository> --branch <branch>, or list it in the host's configuration".into()),
                _ => Err("this host keeps several inboxes: name one with --inbox <name>".into()),
            },
        }
    }
}

/// An inbox as a command reaches it: its name, for the host's
/// directories, and its shared branch.
#[derive(Clone, Debug)]
pub struct Inbox {
    pub name: String,
    pub branch: Branch,
}

impl Inbox {
    /// The inbox of a configured entry: a local repository's branch where
    /// the repository is a path on this host, else the remote's, through
    /// the bare cache (C.2, C.3).
    pub fn of(entry: &InboxEntry) -> Result<Inbox, String> {
        Ok(Inbox {
            name: entry.name.clone(),
            branch: branch(&entry.repository, &entry.branch, &entry.name)?,
        })
    }

    /// An inbox named by its repository and branch on the command line,
    /// in place of the host's configuration (C.1), named for the
    /// repository's path in the host's directories.
    pub fn ad_hoc(repository: &str, name: &str) -> Result<Inbox, String> {
        let hash = content_hash(repository.as_bytes());
        let short: String = hash
            .trim_start_matches("sha256:")
            .chars()
            .take(12)
            .collect();
        Ok(Inbox {
            name: format!("inbox-{short}"),
            branch: branch(repository, name, &short)?,
        })
    }

    /// The host's raw bank for this inbox (C.3).
    pub fn bank_dir(&self) -> PathBuf {
        xdg("XDG_DATA_HOME", ".local/share")
            .join("millpond")
            .join(&self.name)
    }

    /// The host's per-host state for this inbox (C.3).
    pub fn state_dir(&self) -> PathBuf {
        xdg("XDG_STATE_HOME", ".local/state")
            .join("millpond")
            .join(&self.name)
    }
}

fn branch(repository: &str, name: &str, cache: &str) -> Result<Branch, String> {
    let path = Path::new(repository);
    if path.exists() {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
            .output()
            .map_err(|error| format!("git could not be run: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{repository} is no git repository: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Ok(Branch::local(name, git_dir));
    }
    let cache = xdg("XDG_CACHE_HOME", ".cache")
        .join("millpond/git")
        .join(format!("{cache}.git"));
    Ok(Branch::remote(name, cache, repository))
}
