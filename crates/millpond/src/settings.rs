//! An inbox's settings, `.config/millpond.toml` at the root of its branch,
//! read at the version the inbox was read at; millpond never writes them
//! (2; C.2). The policy and the people list are read by weir's crate as
//! it reads them (125); millpond adds the checks C.2 gives it: each level a
//! rule names has its number in `[priorities]`, and a rule that gathers
//! items has `[policy]` naming the host that closes them.

use std::collections::BTreeMap;
use std::time::Duration;

use serde::Serialize;
use toml::{Table, Value};
use weir_cadence::Schedule;
use weir_policy::{Outcome, People, Policy, Priority};

/// Where the settings lie on the inbox's branch.
pub const PATH: &str = ".config/millpond.toml";

/// The limit when the settings give none (C.4).
pub const LIMIT: usize = 8192;

/// How old a lock grows before it is stale, when the settings give none.
pub const LOCK_STALE: Duration = Duration::from_secs(3600);

/// Where an inbox's items go (50).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemsGo {
    Sends,
    Planner,
}

/// One adapter of the inbox (A.1 3; C.2).
#[derive(Clone, Debug)]
pub struct AdapterSetting {
    pub host: String,
    pub schedule: Result<Schedule, String>,
    pub config: serde_json::Value,
}

/// The keeper's policy, as the crates read it, with what C.2 adds.
#[derive(Clone, Debug)]
pub struct Keeper {
    pub policy: Policy,
    pub people: People,
    /// The one host whose tick closes and sends every rule's item (130).
    pub host: Option<String>,
    /// Each level's number in flume's order (131).
    pub priorities: BTreeMap<Priority, i64>,
}

impl Keeper {
    /// The rule named `name`.
    pub fn rule(&self, name: &str) -> Option<&weir_policy::Rule> {
        self.policy.rules().iter().find(|rule| rule.name == name)
    }
}

/// An inbox's settings.
#[derive(Clone, Debug)]
pub struct Settings {
    pub limit: usize,
    pub items_go: ItemsGo,
    pub lock_stale: Duration,
    pub adapters: BTreeMap<String, AdapterSetting>,
    /// The policy, or why it is refused: where it is refused no rule runs
    /// and every signal stays unmoved for curation (125).
    pub keeper: Result<Keeper, String>,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            limit: LIMIT,
            items_go: ItemsGo::Sends,
            lock_stale: LOCK_STALE,
            adapters: BTreeMap::new(),
            keeper: Ok(Keeper {
                policy: Policy::default(),
                people: People::default(),
                host: None,
                priorities: BTreeMap::new(),
            }),
        }
    }
}

/// The fields the settings' top level holds beside weir's tables.
const FIELDS: [&str; 9] = [
    "limit",
    "items",
    "lock_stale",
    "adapters",
    "policy",
    "priorities",
    "rules",
    "person",
    "org",
];

impl Settings {
    /// The settings `text` holds, the defaults where there is none. A
    /// policy refused leaves the rest of the settings standing (125).
    pub fn read(text: Option<&str>) -> Result<Settings, String> {
        let Some(text) = text else {
            return Ok(Settings::default());
        };
        let table: Table = toml::from_str(text).map_err(|error| format!("{PATH}: {error}"))?;
        for key in table.keys() {
            if !FIELDS.contains(&key.as_str()) && key != "about" {
                return Err(format!("{PATH}: `{key}` is no field of the settings"));
            }
        }
        let limit = match table.get("limit") {
            None => LIMIT,
            Some(Value::Integer(n)) if *n > 0 => *n as usize,
            Some(_) => return Err(format!("{PATH}: `limit` is not a count of bytes")),
        };
        let items_go = match table.get("items").and_then(Value::as_str) {
            None | Some("sends") => ItemsGo::Sends,
            Some("planner") => ItemsGo::Planner,
            Some(other) => {
                return Err(format!(
                    "{PATH}: `items` is `{other}`, not `sends` or `planner`"
                ))
            }
        };
        let lock_stale = match table.get("lock_stale") {
            None => LOCK_STALE,
            Some(Value::Integer(n)) if *n > 0 => Duration::from_secs(*n as u64),
            Some(_) => return Err(format!("{PATH}: `lock_stale` is not a count of seconds")),
        };
        let mut adapters = BTreeMap::new();
        if let Some(value) = table.get("adapters") {
            let Value::Table(entries) = value else {
                return Err(format!("{PATH}: `adapters` is not a table"));
            };
            for (name, value) in entries {
                adapters.insert(name.clone(), adapter(name, value)?);
            }
        }
        Ok(Settings {
            limit,
            items_go,
            lock_stale,
            adapters,
            keeper: keeper(text, &table),
        })
    }
}

fn adapter(name: &str, value: &Value) -> Result<AdapterSetting, String> {
    let at = format!("{PATH}: `adapters.{name}`");
    let Value::Table(table) = value else {
        return Err(format!("{at} is not a table"));
    };
    for key in table.keys() {
        if !["host", "schedule", "config"].contains(&key.as_str()) {
            return Err(format!("{at}.{key} is no field of an adapter"));
        }
    }
    let host = table
        .get("host")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{at}.host is missing: an adapter names the one host it runs on"))?
        .to_string();
    let schedule = table
        .get("schedule")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{at}.schedule is missing"))?;
    let schedule = Schedule::parse(schedule).map_err(|error| format!("{at}.schedule: {error}"));
    let config = match table.get("config") {
        None => serde_json::json!({}),
        Some(value) => serde_json::to_value(value).map_err(|error| error.to_string())?,
    };
    Ok(AdapterSetting {
        host,
        schedule,
        config,
    })
}

fn keeper(text: &str, table: &Table) -> Result<Keeper, String> {
    let policy = Policy::read(text).map_err(|error| format!("the policy is refused: {error}"))?;
    let people =
        People::read(text).map_err(|error| format!("the people list is refused: {error}"))?;
    let host = match table.get("policy") {
        None => None,
        Some(Value::Table(policy)) => {
            if let Some(key) = policy.keys().find(|key| key.as_str() != "host") {
                return Err(format!("the policy is refused: `policy.{key}` is no field"));
            }
            match policy.get("host") {
                None => None,
                Some(Value::String(host)) if !host.is_empty() => Some(host.clone()),
                Some(_) => {
                    return Err("the policy is refused: `policy.host` is not a host's name".into())
                }
            }
        }
        Some(_) => return Err("the policy is refused: `policy` is not a table".into()),
    };
    let mut priorities = BTreeMap::new();
    match table.get("priorities") {
        None => {}
        Some(Value::Table(levels)) => {
            for (level, number) in levels {
                let priority = Priority::parse(level).ok_or_else(|| {
                    format!("the policy is refused: `priorities.{level}` names no level")
                })?;
                let Value::Integer(number) = number else {
                    return Err(format!(
                        "the policy is refused: `priorities.{level}` is not a number"
                    ));
                };
                priorities.insert(priority, *number);
            }
        }
        Some(_) => return Err("the policy is refused: `priorities` is not a table".into()),
    }
    for (n, rule) in policy.rules().iter().enumerate() {
        if let Some(level) = rule.priority {
            if !priorities.contains_key(&level) {
                return Err(format!(
                    "the policy is refused: `rules[{n}].priority` is `{level}`, which `[priorities]` gives no number"
                ));
            }
        }
        if !matches!(rule.outcome, Outcome::Drop(_)) && host.is_none() {
            return Err(format!(
                "the policy is refused: `rules[{n}]` gathers items, and `[policy]` names no host to close them"
            ));
        }
    }
    Ok(Keeper {
        policy,
        people,
        host,
        priorities,
    })
}
