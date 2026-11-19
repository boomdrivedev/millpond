//! A scratch world for the binary: a bare repository holding the inbox's
//! branch, a host's configuration and directories of its own, a fake flume
//! and a fake sluice, and a clock fixed by `MILLPOND_NOW`.

#![allow(dead_code)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use serde_json::Value;
use tempfile::TempDir;
use time::OffsetDateTime;
use weir_branch::{Ask, Branch, Files};

pub const HOST: &str = "testhost";
pub const BRANCH: &str = "inbox";

pub struct World {
    pub dir: TempDir,
}

impl World {
    /// A world whose inbox's branch holds `settings` as its settings.
    pub fn new(settings: &str) -> World {
        let world = World {
            dir: tempfile::tempdir().unwrap(),
        };
        let repo = world.repo();
        let status = Command::new("git")
            .args(["init", "--bare", "--quiet"])
            .arg(&repo)
            .status()
            .unwrap();
        assert!(status.success());
        world.seed(&[(".config/millpond.toml", settings)]);
        fs::create_dir_all(world.path("target-repo")).unwrap();
        fs::write(
            world.path("host.toml"),
            format!(
                "host = \"{HOST}\"\nsecrets = \"{}\"\ndeclarations = [\"{}\"]\n\n[[inbox]]\nname = \"family\"\nrepository = \"{}\"\nbranch = \"{BRANCH}\"\n",
                world.path("secrets").display(),
                world.path("adapters/adapter.toml").display(),
                repo.display()
            ),
        )
        .unwrap();
        world.script(
            "fake/flume",
            r#"#!/bin/sh
out="$FAKE_OUT/flume"
mkdir -p "$out"
printf '%s\n' "$*" >> "$out/calls"
printf '%s\n' "$@" > "$out/args"
while [ $# -gt 0 ]; do
  case "$1" in
    --brief-file) cp "$2" "$out/brief" ;;
    --params-file) cp "$2" "$out/params" ;;
  esac
  shift
done
echo '{"place":"made"}'
"#,
        );
        world.script(
            "fake/sluice",
            r#"#!/bin/sh
if [ -f "$FAKE_OUT/sluice.json" ]; then cat "$FAKE_OUT/sluice.json"; else echo '{"submissions":[]}'; fi
"#,
        );
        world
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.dir.path().join(rel)
    }

    pub fn repo(&self) -> PathBuf {
        self.path("inbox.git")
    }

    pub fn branch(&self) -> Branch {
        Branch::local(BRANCH, self.repo())
    }

    /// One write of `files` onto the branch.
    pub fn seed(&self, files: &[(&str, &str)]) {
        let files: Files = files
            .iter()
            .map(|(path, text)| (path.to_string(), text.to_string()))
            .collect();
        let ask = Ask {
            what: "seed",
            who: "test",
            bound: None,
            stale_after: Duration::from_secs(60),
            clock: &OffsetDateTime::now_utc,
        };
        weir_branch::write(&self.branch(), &ask, |_| Ok(files.clone())).unwrap();
    }

    pub fn script(&self, rel: &str, text: &str) -> PathBuf {
        let path = self.path(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// The binary run in this world at `now`, `input` on its standard input.
    pub fn millpond(&self, args: &[&str], now: &str, input: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_millpond"));
        command
            .args(args)
            .env("MILLPOND_CONFIG", self.path("host.toml"))
            .env("MILLPOND_NOW", now)
            .env("MILLPOND_FLUME", self.path("fake/flume"))
            .env("MILLPOND_SLUICE", self.path("fake/sluice"))
            .env("FAKE_OUT", self.path("out"))
            .env("TZ", "UTC")
            .env("HOME", self.path("home"))
            .env("XDG_DATA_HOME", self.path("home/data"))
            .env("XDG_STATE_HOME", self.path("home/state"))
            .env("XDG_CACHE_HOME", self.path("home/cache"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn().unwrap();
        {
            use std::io::Write;
            let mut stdin = child.stdin.take().unwrap();
            stdin.write_all(input.as_bytes()).unwrap();
        }
        child.wait_with_output().unwrap()
    }

    /// The JSON the binary printed last, the run expected to succeed.
    pub fn json(&self, args: &[&str], now: &str, input: &str) -> Value {
        let output = self.millpond(args, now, input);
        assert!(
            output.status.success(),
            "millpond {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        let last = stdout.lines().last().unwrap_or_default();
        serde_json::from_str(last).unwrap_or_else(|error| panic!("{error}: {stdout}"))
    }

    pub fn out(&self, rel: &str) -> Option<String> {
        fs::read_to_string(self.path("out").join(rel)).ok()
    }

    pub fn file_at_tip(&self, path: &str) -> Option<String> {
        weir_branch::read(&self.branch())
            .unwrap()
            .and_then(|tip| tip.file(path).unwrap())
    }
}

/// A capture of a release as headwater offers one (C.12): its key from its
/// Package URL, its notes as raw material and its own excerpt, and its
/// facts.
pub fn release_line(name: &str, version: &str, pin: &str, event_at: &str) -> String {
    let key = format!("release/github/boomdrivedev/{name}/{version}");
    serde_json::json!({"capture": {
        "key": key,
        "source": "release",
        "event_at": event_at,
        "title": format!("{name} {version}"),
        "url": format!("https://github.com/boomdrivedev/{name}/releases/tag/v{version}"),
        "raw": [format!("{key}/notes.md")],
        "excerpt": {
            "text": format!("## {name} v{version}\n\n- what changed\n"),
            "kind": "ask",
            "asserted_by": format!("boomdrivedev/{name}"),
            "assertion": format!("{name} {version} is released; the repository pins {pin}."),
            "subjects": ["release", name],
            "argues_with": ["flake.lock"],
            "position": "release notes",
        },
        "facts": {
            "identity": format!("pkg:github/boomdrivedev/{name}"),
            "pin": pin,
            "version": version,
            "level": "minor",
        },
    }})
    .to_string()
}

/// Writes the notes the line names under `raw`.
pub fn write_notes(raw: &Path, name: &str, version: &str) {
    let path = raw.join(format!(
        "release/github/boomdrivedev/{name}/{version}/notes.md"
    ));
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!("## {name} v{version}\n\n- what changed\n")).unwrap();
}

/// The settings of the family's rule: a release of the family's own tools
/// launches `kind` in `repository` at normal priority over a ten-minute
/// window.
pub fn family_settings(kind: &str, repository: &Path) -> String {
    format!(
        r#"[policy]
host = "{HOST}"

[priorities]
low = 10
normal = 20
elevated = 30
high = 40
urgent = 50

[[rules]]
name = "family"
when = {{ fact = "identity", contains = "pkg:github/boomdrivedev/" }}
kind = "{kind}"
repository = "{}"
priority = "normal"
window = "*/10 * * * *"
"#,
        repository.display()
    )
}
