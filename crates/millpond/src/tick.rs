//! A tick: one pass over an inbox on this host (28, 117, 130; C.1). It
//! runs every adapter of the inbox that names this host and is due; reads
//! the merge queue's record of the branch of each open send to a place,
//! writing a landing once; and, on the host the settings name for the
//! policy, closes each rule's item whose window has come, writing its
//! brief of its signals' excerpts, and sends each closed item of a rule
//! that names a kind to a place of that kind in the rule's repository.

use jiff::Timestamp;
use serde_json::{json, Value};
use weir_cadence::is_due;
use weir_policy::Outcome as RuleOutcome;
use weir_signals::{InboxAt, Outcome};

use crate::clock;
use crate::host::HostConfig;
use crate::items;
use crate::run;
use crate::send::{self, Launch};
use crate::settings::{ItemsGo, Settings};
use crate::state;
use crate::store;
use crate::take::Taker;

/// One tick, and what it did, as JSON.
pub fn tick(taker: &Taker, host: &HostConfig) -> Result<Value, String> {
    let mut errors: Vec<String> = Vec::new();
    let (_, _, settings) = store::read(&taker.inbox.branch)?;
    let ran = run_due(taker, host, &settings);
    let landed = read_landings(taker, &settings, &mut errors)?;
    let (closed, sent, policy) = match &settings.keeper {
        Err(why) => (Vec::new(), Vec::new(), json!({"refused": why})),
        Ok(keeper) if keeper.host.as_deref() == Some(taker.machine) => {
            let closed = close_due(taker, &settings, &mut errors)?;
            let sent = match settings.items_go {
                ItemsGo::Sends => send_closed(taker, &settings, &mut errors)?,
                ItemsGo::Planner => Vec::new(),
            };
            (closed, sent, json!({"host": keeper.host}))
        }
        Ok(keeper) => (Vec::new(), Vec::new(), json!({"host": keeper.host})),
    };
    Ok(json!({
        "inbox": taker.inbox.name,
        "host": taker.machine,
        "at": clock::stamp(),
        "ran": ran,
        "landed": landed,
        "policy": policy,
        "closed": closed,
        "sent": sent,
        "errors": errors,
    }))
}

/// Runs every adapter that names this host and is due: never run here, or
/// a time its schedule names has come since its last run, missed times
/// caught up once (28, 29; C.4).
fn run_due(taker: &Taker, host: &HostConfig, settings: &Settings) -> Vec<Value> {
    let now = clock::zoned();
    let dir = taker.inbox.state_dir();
    settings
        .adapters
        .iter()
        .filter(|(_, setting)| setting.host == taker.machine)
        .filter_map(|(name, setting)| {
            let schedule = match &setting.schedule {
                Ok(schedule) => schedule,
                Err(why) => return Some(json!({"adapter": name, "not_run": why})),
            };
            let last = state::read(&dir, name)
                .status
                .and_then(|status| status.ran_at.parse::<Timestamp>().ok());
            if !is_due(schedule, last, &now) {
                return None;
            }
            Some(match run::configured(taker, host, name) {
                Ok(report) => json!({
                    "adapter": name,
                    "report": serde_json::from_str::<Value>(&report.to_json()).ok(),
                }),
                Err(why) => json!({"adapter": name, "error": why}),
            })
        })
        .collect()
}

/// Each open send to a place, by item, target and time.
fn open_sends(inbox: &InboxAt) -> Vec<(String, String, String)> {
    items::names(inbox)
        .into_iter()
        .filter_map(|name| inbox.item(&name).map(|at| (name, at)))
        .flat_map(|(name, at)| {
            let settled: Vec<(String, String)> = at
                .sends
                .iter()
                .filter(|record| !record.is_send())
                .map(|record| (record.target.clone(), record.sent_at.clone()))
                .collect();
            at.sends
                .iter()
                .filter(|record| record.is_send() && send::place_of(&record.target).is_some())
                .filter(|record| {
                    !settled.contains(&(record.target.clone(), record.sent_at.clone()))
                })
                .map(|record| (name.clone(), record.target.clone(), record.sent_at.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Reads sluice's record of each open send's branch, and writes a landing
/// where the branch's work landed, once (55, 117).
fn read_landings(
    taker: &Taker,
    settings: &Settings,
    errors: &mut Vec<String>,
) -> Result<Vec<Value>, String> {
    let (_, inbox, _) = store::read(&taker.inbox.branch)?;
    let mut landed = Vec::new();
    for (item, target, sent_at) in open_sends(&inbox) {
        let Some((_, name, repository)) = send::place_of(&target) else {
            continue;
        };
        match send::landed(repository, name) {
            Ok(None) => {}
            Ok(Some((onto, revision))) => {
                let outcome = Outcome::Landing(format!("{onto} {revision}"));
                let written = store::write(
                    &taker.inbox.branch,
                    &format!("record {item}'s landing"),
                    &store::who(taker.machine),
                    settings.lock_stale,
                    |inbox| {
                        inbox
                            .settle_send(&item, &target, &sent_at, &outcome)
                            .map(Some)
                            .map_err(|refused| refused.to_string())
                    },
                );
                match written {
                    Ok(_) => landed.push(json!({"item": item, "onto": onto, "revision": revision})),
                    Err(why) => errors.push(format!("{item}'s landing: {why}")),
                }
            }
            Err(why) => errors.push(format!("{item}'s landing: {why}")),
        }
    }
    Ok(landed)
}

/// Closes each rule's item whose window has come: a time the window names
/// since the item was made, or at once for a rule with no window (130;
/// weir 129).
fn close_due(
    taker: &Taker,
    settings: &Settings,
    errors: &mut Vec<String>,
) -> Result<Vec<String>, String> {
    let Ok(keeper) = &settings.keeper else {
        return Ok(Vec::new());
    };
    let (_, inbox, _) = store::read(&taker.inbox.branch)?;
    let now = clock::zoned();
    let mut closed = Vec::new();
    for name in items::names(&inbox) {
        let Some(at) = inbox.item(&name) else {
            continue;
        };
        let (Some(rule), None) = (&at.item.rule, &at.item.closed_at) else {
            continue;
        };
        let Some(rule) = keeper.rule(rule) else {
            continue;
        };
        let due = match &rule.window {
            None => true,
            Some(window) => is_due(window, at.item.named_at.parse().ok(), &now),
        };
        if !due {
            continue;
        }
        let closed_at = clock::stamp();
        let written = store::write(
            &taker.inbox.branch,
            &format!("close {name}"),
            &store::who(taker.machine),
            settings.lock_stale,
            |inbox| {
                inbox
                    .close_rule_item(&name, &closed_at)
                    .map(Some)
                    .map_err(|refused| refused.to_string())
            },
        );
        match written {
            Ok(_) => closed.push(name),
            Err(why) => errors.push(format!("{name}'s close: {why}")),
        }
    }
    Ok(closed)
}

/// Sends each closed item of a rule that names a kind, with a brief and no
/// send yet, to a place of that kind in the rule's repository: the launch
/// first, then the send's record (52, 53, 115, 130, 131). An item of a
/// person's turn stays proposed, for a person to send.
fn send_closed(
    taker: &Taker,
    settings: &Settings,
    errors: &mut Vec<String>,
) -> Result<Vec<Value>, String> {
    let Ok(keeper) = &settings.keeper else {
        return Ok(Vec::new());
    };
    let (_, inbox, _) = store::read(&taker.inbox.branch)?;
    let mut sent = Vec::new();
    for name in items::names(&inbox) {
        let Some(at) = inbox.item(&name) else {
            continue;
        };
        let (Some(rule_name), Some(_), Some(brief)) =
            (&at.item.rule, &at.item.closed_at, &at.brief)
        else {
            continue;
        };
        if at.sends.iter().any(|record| record.is_send()) {
            continue;
        }
        let Some(rule) = keeper.rule(rule_name) else {
            continue;
        };
        let RuleOutcome::Launch { kind, repository } = &rule.outcome else {
            continue;
        };
        let Some(priority) = rule
            .priority
            .and_then(|level| keeper.priorities.get(&level))
        else {
            continue;
        };
        let n = 1;
        let launch = Launch {
            kind: kind.clone(),
            name: format!("{name}--{n}"),
            repository: repository.clone(),
            brief: brief.clone(),
            params: items::params(&inbox, &name),
            priority: *priority,
            mark: format!(
                "millpond {} {name} send {n} asked by rule:{rule_name}",
                taker.inbox.name
            ),
        };
        let answer = match launch.run() {
            Ok(answer) => answer,
            Err(why) => {
                errors.push(why);
                continue;
            }
        };
        let target = launch.target();
        let sent_at = clock::stamp();
        let written = store::write(
            &taker.inbox.branch,
            &format!("send {name} to {}", launch.name),
            &store::who(taker.machine),
            settings.lock_stale,
            |inbox| {
                inbox
                    .send(&[name.as_str()], &target, &sent_at)
                    .map(Some)
                    .map_err(|refused| refused.to_string())
            },
        );
        match written {
            Ok(_) => sent.push(json!({
                "item": name,
                "target": target,
                "launch": launch.args(std::path::Path::new("<brief>"), std::path::Path::new("<params>")),
                "params": launch.params,
                "flume": serde_json::from_str::<Value>(&answer).unwrap_or(Value::String(answer)),
            })),
            Err(why) => errors.push(format!("{name}'s send: {why}")),
        }
    }
    Ok(sent)
}
