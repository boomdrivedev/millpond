//! The `millpond` command line: one caller of the core (64), printing what
//! a program reads as JSON (65).

use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use serde_json::{json, Value};
use weir_capture::CameIn;

use millpond::host::{HostConfig, Inbox};
use millpond::run;
use millpond::state;
use millpond::store;
use millpond::take::Taker;
use millpond::{items, tick};

const VERSION: &str = concat!(
    "revision ",
    env!("MILLPOND_REVISION"),
    ", built by ",
    env!("MILLPOND_BUILD")
);

#[derive(Parser)]
#[command(name = "millpond", version = VERSION, about = "The inbox for events that should change a repository's work")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Which inbox: one the host's configuration names, or a repository and
/// its branch named here in place of it (C.1).
#[derive(Args, Clone)]
struct Which {
    /// The inbox, by its name in the host's configuration; with --branch,
    /// a repository's path or address
    #[arg(long)]
    inbox: Option<String>,
    /// The inbox's branch, with --inbox naming its repository
    #[arg(long)]
    branch: Option<String>,
    /// The limit in bytes, in place of the settings'
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Subcommand)]
enum Command {
    /// The intake: take capture-contract lines on standard input (27), or
    /// run a declared adapter (--run) or import (--import), and print
    /// weir-report/1 last
    Land {
        #[command(flatten)]
        which: Which,
        /// The folder the lines' raw files are in (106)
        #[arg(long, conflicts_with_all = ["run", "import"])]
        raw_dir: Option<PathBuf>,
        /// Run the one adapter this declaration file declares, as a run
        #[arg(long, conflicts_with = "import")]
        run: Option<PathBuf>,
        /// Take weir's import lines on standard input (60 to 63)
        #[arg(long)]
        import: bool,
    },
    /// Run one adapter of the inbox on this host, as its settings say
    Run {
        adapter: String,
        #[command(flatten)]
        which: Which,
    },
    /// One pass over the inbox on this host: due adapters, landings, and on
    /// the policy's host each due rule's item closed and sent
    Tick {
        #[command(flatten)]
        which: Which,
    },
    /// Each adapter's status on this host, the policy's, and the count and
    /// age of unmoved signals by source
    Status {
        #[command(flatten)]
        which: Which,
        /// Print JSON (the only form)
        #[arg(long)]
        json: bool,
    },
    /// The inbox's items, with their signals, weight, briefs and sends
    Items {
        #[command(flatten)]
        which: Which,
        /// Print JSON (the only form)
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match command(cli.command) {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(why) => {
            eprintln!("millpond: {}", why.replace('\n', " "));
            ExitCode::from(1)
        }
    }
}

/// The inbox `which` names, this host's configuration, and its machine.
fn reach(which: &Which) -> Result<(Inbox, HostConfig, String), String> {
    let host = HostConfig::load()?;
    let machine = host.machine()?;
    let inbox = match (&which.inbox, &which.branch) {
        (Some(repository), Some(branch)) => Inbox::ad_hoc(repository, branch)?,
        (None, Some(_)) => {
            return Err("--branch names the branch of the repository --inbox names".into())
        }
        (name, None) => Inbox::of(host.inbox(name.as_deref())?)?,
    };
    Ok((inbox, host, machine))
}

fn stdin_lines() -> Result<Vec<String>, String> {
    io::stdin()
        .lock()
        .lines()
        .collect::<Result<_, _>>()
        .map_err(|error| format!("standard input: {error}"))
}

fn command(command: Command) -> Result<Value, String> {
    match command {
        Command::Land {
            which,
            raw_dir,
            run: declaration,
            import,
        } => {
            let (inbox, host, machine) = reach(&which)?;
            let (_, _, settings) = store::read(&inbox.branch)?;
            let taker = Taker {
                inbox: &inbox,
                machine: &machine,
                limit: which.limit.unwrap_or(settings.limit),
            };
            let report = if let Some(path) = declaration {
                let declared = run::declared(&path, None)?
                    .ok_or_else(|| format!("{} declares no adapter", path.display()))?;
                run::run(
                    &taker,
                    &declared,
                    &json!({}),
                    host.secrets.as_deref(),
                    settings.lock_stale,
                )?
            } else if import {
                taker.import(&stdin_lines()?)?
            } else {
                taker
                    .take_lines(&stdin_lines()?, raw_dir.as_deref(), CameIn::Intake)?
                    .finish(true)
            };
            Ok(serde_json::from_str(&report.to_json()).map_err(|e| e.to_string())?)
        }
        Command::Run { adapter, which } => {
            let (inbox, host, machine) = reach(&which)?;
            let (_, _, settings) = store::read(&inbox.branch)?;
            let taker = Taker {
                inbox: &inbox,
                machine: &machine,
                limit: which.limit.unwrap_or(settings.limit),
            };
            let report = run::configured(&taker, &host, &adapter)?;
            Ok(serde_json::from_str(&report.to_json()).map_err(|e| e.to_string())?)
        }
        Command::Tick { which } => {
            let (inbox, host, machine) = reach(&which)?;
            let (_, _, settings) = store::read(&inbox.branch)?;
            let taker = Taker {
                inbox: &inbox,
                machine: &machine,
                limit: which.limit.unwrap_or(settings.limit),
            };
            tick::tick(&taker, &host)
        }
        Command::Status { which, .. } => {
            let (inbox, _, machine) = reach(&which)?;
            let (_, at, settings) = store::read(&inbox.branch)?;
            let dir = inbox.state_dir();
            let adapters: serde_json::Map<String, Value> = settings
                .adapters
                .iter()
                .filter(|(_, setting)| setting.host == machine)
                .map(|(name, _)| {
                    let kept = state::read(&dir, name);
                    (
                        name.clone(),
                        json!({"cursor": kept.cursor, "status": kept.status}),
                    )
                })
                .collect();
            let policy = match &settings.keeper {
                Ok(keeper) => json!({"rules": keeper.policy.rules().len(), "host": keeper.host}),
                Err(why) => json!({"refused": why}),
            };
            Ok(json!({
                "inbox": inbox.name,
                "host": machine,
                "adapters": adapters,
                "policy": policy,
                "unmoved": items::unmoved(&at),
            }))
        }
        Command::Items { which, .. } => {
            let (inbox, _, _) = reach(&which)?;
            let (_, at, _) = store::read(&inbox.branch)?;
            Ok(items::listing(&at))
        }
    }
}
