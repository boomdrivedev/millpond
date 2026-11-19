//! What an inbox's records say of its items and signals, read from one
//! tip, as JSON (45, 49, 64 to 67; A.12). Nothing here writes.

use std::collections::BTreeMap;

use jiff::Timestamp;
use serde_json::{json, Value};
use weir_signals::{InboxAt, MadeBy, Outcome};

/// Every item's name, as the layout places its record.
pub fn names(inbox: &InboxAt) -> Vec<String> {
    inbox
        .files()
        .keys()
        .filter_map(|path| {
            path.strip_prefix("inbox/items/")?
                .strip_suffix("/item.rec")
                .map(str::to_string)
        })
        .collect()
}

/// The stated facts of each of the item's signals' captures, one entry
/// per signal in the order of their event times (C.5).
pub fn params(inbox: &InboxAt, item: &str) -> Value {
    let mut entries: Vec<(Option<Timestamp>, String, Value)> = inbox
        .item_signals(item)
        .into_iter()
        .filter_map(|id| {
            let capture = inbox.capture(&id.key)?;
            Some((
                capture.event_at.parse().ok(),
                id.to_string(),
                capture.facts.to_json(),
            ))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    Value::Array(entries.into_iter().map(|(_, _, facts)| facts).collect())
}

/// The item `name` as a listing gives it: its rule and priority, whether
/// closed, its signals, weight, brief and sends with what became of each.
pub fn item_json(inbox: &InboxAt, name: &str) -> Option<Value> {
    let at = inbox.item(name)?;
    let signals: Vec<String> = inbox
        .item_signals(name)
        .iter()
        .map(|id| id.to_string())
        .collect();
    let weight = inbox.weight(name).map(|weight| {
        json!({
            "signals": weight.signals,
            "sources": weight.sources,
            "earliest": weight.earliest,
            "latest": weight.latest,
        })
    });
    let mut settled: BTreeMap<(String, String), String> = BTreeMap::new();
    for record in at.sends.iter().filter(|record| !record.is_send()) {
        settled.insert(
            (record.target.clone(), record.sent_at.clone()),
            record.outcome.to_string(),
        );
    }
    let sends: Vec<Value> = at
        .sends
        .iter()
        .filter(|record| record.is_send())
        .map(|record| {
            let outcome = settled
                .get(&(record.target.clone(), record.sent_at.clone()))
                .cloned()
                .unwrap_or_else(|| Outcome::Open.to_string());
            json!({
                "target": record.target,
                "sent_at": record.sent_at,
                "outcome": outcome,
            })
        })
        .collect();
    Some(json!({
        "item": name,
        "rule": at.item.rule,
        "priority": at.item.priority.map(|p| p.as_str()),
        "named_by": at.item.named_by,
        "named_at": at.item.named_at,
        "closed_at": at.item.closed_at,
        "state": if at.is_sent() { "sent" } else { "proposed" },
        "signals": signals,
        "weight": weight,
        "brief": at.brief,
        "sends": sends,
    }))
}

/// Every item with signals, or made from a conclusion, as a listing gives
/// it; an item left with no signals is left out (114).
pub fn listing(inbox: &InboxAt) -> Value {
    Value::Array(
        names(inbox)
            .iter()
            .filter(|name| {
                !inbox.item_signals(name).is_empty()
                    || inbox
                        .item(name)
                        .is_some_and(|at| matches!(at.item.made_by, MadeBy::Conclusion(_)))
            })
            .filter_map(|name| item_json(inbox, name))
            .collect(),
    )
}

/// The count and the oldest event time of unmoved signals by source (67).
pub fn unmoved(inbox: &InboxAt) -> Value {
    let mut by_source: BTreeMap<String, (usize, String)> = BTreeMap::new();
    for unmoved in inbox.unmoved() {
        let source = inbox
            .capture(&unmoved.signal.signal.key)
            .map(|capture| capture.source.clone())
            .unwrap_or_default();
        let entry = by_source
            .entry(source)
            .or_insert((0, unmoved.event_at.to_string()));
        entry.0 += 1;
        let older = match (
            unmoved.event_at.parse::<Timestamp>(),
            entry.1.parse::<Timestamp>(),
        ) {
            (Ok(this), Ok(oldest)) => this < oldest,
            _ => false,
        };
        if older {
            entry.1 = unmoved.event_at.to_string();
        }
    }
    Value::Object(
        by_source
            .into_iter()
            .map(|(source, (count, oldest))| (source, json!({"count": count, "oldest": oldest})))
            .collect(),
    )
}
