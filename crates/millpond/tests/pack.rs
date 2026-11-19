//! millpond held to weir's conformance pack as a host (27, 102; C.1): each
//! host case of the pack run against the binary through the pack's seam,
//! `--inbox <repo> --branch inbox --limit <bytes>` and the step's flags,
//! its printed report and its branch's tip compared by weir-pack.

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use time::OffsetDateTime;
use weir_branch::{write, Ask, Branch, Files};
use weir_pack::{
    compare_report, compare_tip, read_host_case, Case, HostCase, Invocation, Machine, Pack, Shape,
};

const BRANCH: &str = "inbox";
const HOST: &str = "packhost";

fn git(args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .env("GIT_AUTHOR_NAME", "pack")
        .env("GIT_AUTHOR_EMAIL", "")
        .env("GIT_COMMITTER_NAME", "pack")
        .env("GIT_COMMITTER_EMAIL", "")
        .stdin(Stdio::null())
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

fn seed(branch: &Branch, repo: &Path, start: &BTreeMap<String, Vec<u8>>) {
    let files: Files = start
        .iter()
        .map(|(path, bytes)| (path.clone(), String::from_utf8(bytes.clone()).unwrap()))
        .collect();
    if files.is_empty() {
        let tree = Command::new("git")
            .arg("--git-dir")
            .arg(repo)
            .args(["mktree"])
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let tree = String::from_utf8_lossy(&tree.stdout).trim().to_string();
        let commit = Command::new("git")
            .arg("--git-dir")
            .arg(repo)
            .args(["commit-tree", "--no-gpg-sign", &tree, "-m", "seed"])
            .env("GIT_AUTHOR_NAME", "pack")
            .env("GIT_AUTHOR_EMAIL", "")
            .env("GIT_COMMITTER_NAME", "pack")
            .env("GIT_COMMITTER_EMAIL", "")
            .output()
            .unwrap();
        let commit = String::from_utf8_lossy(&commit.stdout).trim().to_string();
        git(&[
            "--git-dir",
            repo.to_str().unwrap(),
            "update-ref",
            &format!("refs/heads/{BRANCH}"),
            &commit,
            "",
        ]);
        return;
    }
    let ask = Ask {
        what: "seed the case's starting records",
        who: "pack",
        bound: None,
        stale_after: Duration::from_secs(60),
        clock: &OffsetDateTime::now_utc,
    };
    write(branch, &ask, |_| Ok(files.clone())).unwrap();
}

fn declaration(scratch: &Path, name: &str, script: &Path) -> PathBuf {
    fs::write(scratch.join("config.schema.json"), "{}\n").unwrap();
    let path = scratch.join("adapter.toml");
    fs::write(
        &path,
        format!(
            "[adapter.{name:?}]\ncommand = [{:?}]\nconfig = \"config.schema.json\"\nsecrets = []\n",
            script.to_string_lossy()
        ),
    )
    .unwrap();
    path
}

fn read_place(path: &str) -> Result<Vec<Vec<u8>>, String> {
    fn walk(path: &Path, found: &mut Vec<Vec<u8>>) -> io::Result<()> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                walk(&entry?.path(), found)?;
            }
        } else {
            found.push(fs::read(path)?);
        }
        Ok(())
    }
    let mut found = Vec::new();
    walk(Path::new(path), &mut found).map_err(|error| error.to_string())?;
    Ok(found)
}

fn run_step(
    case: &HostCase,
    invocation: &Invocation,
    repo: &Path,
    case_dir: &Path,
    scratch: &Path,
) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_millpond"));
    command
        .arg("land")
        .arg("--inbox")
        .arg(repo)
        .args(["--branch", BRANCH, "--limit", &case.limit.to_string()])
        .env("MILLPOND_CONFIG", scratch.join("host.toml"))
        .env("XDG_DATA_HOME", scratch.join("data"))
        .env("XDG_STATE_HOME", scratch.join("state"))
        .env("XDG_CACHE_HOME", scratch.join("cache"));
    let input = match invocation {
        Invocation::Intake { lines, raw } => {
            if let Some(raw) = raw {
                command.arg("--raw-dir").arg(case_dir.join(raw));
            }
            Some(case_dir.join(lines))
        }
        Invocation::Run { adapter, .. } => {
            let declared = declaration(scratch, &case.name, &case_dir.join(adapter));
            command.arg("--run").arg(declared);
            None
        }
        Invocation::Import { lines } => {
            command.arg("--import");
            Some(case_dir.join(lines))
        }
    };
    command.stdin(match input {
        Some(path) => Stdio::from(File::open(path).unwrap()),
        None => Stdio::null(),
    });
    let output = command.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "no report: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
        })
}

fn run_case(case: &HostCase) -> Vec<String> {
    let scratch = tempfile::tempdir().unwrap();
    fs::write(
        scratch.path().join("host.toml"),
        format!("host = \"{HOST}\"\n"),
    )
    .unwrap();
    let repo = scratch.path().join("inbox.git");
    git(&["init", "--bare", "--quiet", repo.to_str().unwrap()]);
    let branch = Branch::local(BRANCH, &repo);
    seed(&branch, &repo, &case.start);
    let case_dir = scratch.path().join("case");
    Case {
        shape: Shape::Host,
        name: case.name.clone(),
        files: case.files.clone(),
    }
    .write_to(&case_dir)
    .unwrap();
    let mut said = Vec::new();
    let began = OffsetDateTime::now_utc();
    for (n, step) in (1..).zip(&case.steps) {
        let printed = run_step(case, &step.invocation, &repo, &case_dir, scratch.path());
        said.extend(
            compare_report(&step.expected, &printed)
                .into_iter()
                .map(|difference| format!("step {n}: {difference}")),
        );
    }
    let ended = OffsetDateTime::now_utc();
    let tip = weir_branch::read(&branch)
        .unwrap()
        .map(|tip| {
            tip.files("")
                .unwrap()
                .into_iter()
                .map(|(path, text)| (path, text.into_bytes()))
                .collect()
        })
        .unwrap_or_default();
    said.extend(compare_tip(
        case,
        &tip,
        &Machine {
            hostname: HOST,
            began,
            ended,
            read: &read_place,
        },
    ));
    said
}

#[test]
fn millpond_passes_every_host_case_of_the_pack() {
    let pack = Pack::embedded();
    let cases = pack.cases(Shape::Host);
    assert!(!cases.is_empty());
    let failures: Vec<String> = cases
        .iter()
        .flat_map(|case| {
            let host_case = read_host_case(case).unwrap();
            run_case(&host_case)
                .into_iter()
                .map(move |difference| format!("{}: {difference}", case.name))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
