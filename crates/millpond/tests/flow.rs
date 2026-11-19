//! The must-work path, end to end through the binary: a release lands
//! once (9, 27), the family's rule gathers it into its item as it lands
//! (127 to 130), and at the first tick after the window closes the item is
//! closed with its brief and sent to a place of the rule's kind through
//! flume's launch (52, 115, 130, 131; C.5); the place's landing is read
//! from sluice's record at a later tick (55, 117; C.7).

mod common;

use common::{family_settings, release_line, write_notes, World};
use serde_json::Value;

const LANDED_AT: &str = "2026-11-19T10:03:00Z";
const ITEM: &str = "family-20261119T101000Z";

/// A world with the family's rule, and sluice 0.97.0's release landed.
fn landed() -> World {
    let world = World::new("");
    let settings = family_settings("boomdrive-update", &world.path("target-repo"));
    world.seed(&[(".config/millpond.toml", &settings)]);
    let raw = world.path("raw");
    write_notes(&raw, "sluice", "0.97.0");
    let line = release_line("sluice", "0.97.0", "0.95.1", "2026-11-19T09:58:00Z");
    let report = world.json(
        &["land", "--raw-dir", raw.to_str().unwrap()],
        LANDED_AT,
        &line,
    );
    assert_eq!(
        report["taken"],
        serde_json::json!(["release/github/boomdrivedev/sluice/0.97.0"])
    );
    world
}

#[test]
fn a_release_lands_once_and_its_rule_gathers_it() {
    let world = landed();
    let raw = world.path("raw");
    let line = release_line("sluice", "0.97.0", "0.95.1", "2026-11-19T09:58:00Z");
    let again = world.json(
        &["land", "--raw-dir", raw.to_str().unwrap()],
        "2026-11-19T10:04:00Z",
        &line,
    );
    assert_eq!(again["taken"], serde_json::json!([]));
    assert_eq!(again["held"], 1);
    let items = world.json(&["items", "--json"], "2026-11-19T10:04:00Z", "");
    let items = items.as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item"], ITEM);
    assert_eq!(items[0]["rule"], "family");
    assert_eq!(items[0]["priority"], "normal");
    assert_eq!(items[0]["state"], "proposed");
    assert_eq!(
        items[0]["signals"],
        serde_json::json!(["release/github/boomdrivedev/sluice/0.97.0#1"])
    );
    let moves = world.file_at_tip("inbox/moves/moves.rec").unwrap();
    assert!(moves.contains("by: rule:family"), "{moves}");
    let banked = world.path(
        "home/data/millpond/family/release/github/boomdrivedev/sluice/0.97.0/release/github/boomdrivedev/sluice/0.97.0/notes.md",
    );
    assert!(banked.exists(), "{}", banked.display());
}

#[test]
fn a_window_not_closed_sends_nothing() {
    let world = landed();
    let tick = world.json(&["tick"], "2026-11-19T10:05:00Z", "");
    assert_eq!(tick["closed"], serde_json::json!([]));
    assert_eq!(tick["sent"], serde_json::json!([]));
    assert!(world.out("flume/calls").is_none());
}

#[test]
fn the_first_tick_after_the_window_launches_the_rules_kind() {
    let world = landed();
    let line = release_line("flume", "0.30.0", "0.29.0", "2026-11-19T10:06:00Z");
    let raw = world.path("raw");
    write_notes(&raw, "flume", "0.30.0");
    world.json(
        &["land", "--raw-dir", raw.to_str().unwrap()],
        "2026-11-19T10:07:00Z",
        &line,
    );
    let tick = world.json(&["tick"], "2026-11-19T10:10:30Z", "");
    assert_eq!(tick["errors"], serde_json::json!([]), "{tick}");
    assert_eq!(tick["closed"], serde_json::json!([ITEM]));
    let args: Vec<String> = world
        .out("flume/args")
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    let after = |flag: &str| {
        let at = args.iter().position(|arg| arg == flag).unwrap();
        args[at + 1].clone()
    };
    assert_eq!(args[0], "launch");
    assert_eq!(after("--kind"), "boomdrive-update");
    assert_eq!(after("--name"), format!("{ITEM}--1"));
    assert_eq!(
        after("--repository"),
        world.path("target-repo").display().to_string()
    );
    assert_eq!(after("--priority"), "20");
    assert!(after("--mark").contains(ITEM));
    assert!(after("--mark").contains("rule:family"));
    assert_eq!(
        world.out("flume/brief").unwrap(),
        "## sluice v0.97.0\n\n- what changed\n\n\n## flume v0.30.0\n\n- what changed\n"
    );
    let params: Value = serde_json::from_str(&world.out("flume/params").unwrap()).unwrap();
    assert_eq!(params[0]["identity"], "pkg:github/boomdrivedev/sluice");
    assert_eq!(params[0]["version"], "0.97.0");
    assert_eq!(params[0]["pin"], "0.95.1");
    assert_eq!(params[1]["identity"], "pkg:github/boomdrivedev/flume");
    let items = world.json(&["items"], "2026-11-19T10:11:00Z", "");
    let item = &items.as_array().unwrap()[0];
    assert_eq!(item["state"], "sent");
    assert_eq!(item["sends"][0]["outcome"], "open");
    // A second tick sends nothing again (115), and a later release joins
    // the rule's next item (130).
    let again = world.json(&["tick"], "2026-11-19T10:12:00Z", "");
    assert_eq!(again["sent"], serde_json::json!([]));
    assert_eq!(world.out("flume/calls").unwrap().lines().count(), 1);
    write_notes(&raw, "timber", "0.10.0");
    world.json(
        &["land", "--raw-dir", raw.to_str().unwrap()],
        "2026-11-19T10:13:00Z",
        &release_line("timber", "0.10.0", "0.9.0", "2026-11-19T10:12:30Z"),
    );
    let items = world.json(&["items"], "2026-11-19T10:14:00Z", "");
    let names: Vec<&str> = items
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["item"].as_str().unwrap())
        .collect();
    assert_eq!(names, [ITEM, "family-20261119T102000Z"]);
}

#[test]
fn a_tick_records_the_places_landing_from_sluice() {
    let world = landed();
    world.json(&["tick"], "2026-11-19T10:10:30Z", "");
    std::fs::write(
        world.path("out/sluice.json"),
        serde_json::json!({"submissions": [{
            "branch": "main",
            "from": {"head": format!("update/{ITEM}--1")},
            "state": "landed",
            "landed_as": "abc1234",
        }]})
        .to_string(),
    )
    .unwrap();
    let tick = world.json(&["tick"], "2026-11-19T10:20:30Z", "");
    assert_eq!(tick["landed"][0]["item"], ITEM, "{tick}");
    let items = world.json(&["items"], "2026-11-19T10:21:00Z", "");
    assert_eq!(items[0]["sends"][0]["outcome"], "landing main abc1234");
    // Read once: the next tick reads nothing more (55).
    let tick = world.json(&["tick"], "2026-11-19T10:30:30Z", "");
    assert_eq!(tick["landed"], serde_json::json!([]));
}

#[test]
fn a_level_with_no_number_refuses_the_policy() {
    let world = World::new("");
    let settings = family_settings("boomdrive-update", &world.path("target-repo"))
        .replace("normal = 20\n", "");
    world.seed(&[(".config/millpond.toml", &settings)]);
    let raw = world.path("raw");
    write_notes(&raw, "sluice", "0.97.0");
    world.json(
        &["land", "--raw-dir", raw.to_str().unwrap()],
        LANDED_AT,
        &release_line("sluice", "0.97.0", "0.95.1", "2026-11-19T09:58:00Z"),
    );
    let status = world.json(&["status", "--json"], LANDED_AT, "");
    let why = status["policy"]["refused"].as_str().unwrap();
    assert!(why.contains("rules[0].priority"), "{why}");
    assert_eq!(status["unmoved"]["release"]["count"], 1);
    assert_eq!(world.json(&["items"], LANDED_AT, ""), serde_json::json!([]));
}

#[test]
fn a_configured_adapter_runs_and_keeps_its_cursor() {
    let world = World::new("");
    let script = world.script(
        "adapters/releases.sh",
        &format!(
            "#!/bin/sh\nraw=\"$2\"\nmkdir -p \"$raw/release/github/boomdrivedev/sluice/0.97.0\"\nprintf '## sluice v0.97.0\\n\\n- what changed\\n' > \"$raw/release/github/boomdrivedev/sluice/0.97.0/notes.md\"\ncat <<'LINES'\n{}\n{{\"cursor\": \"2026-11-19\"}}\n{{\"done\": {{\"listed\": 1, \"failed\": []}}}}\nLINES\n",
            release_line("sluice", "0.97.0", "0.95.1", "2026-11-19T09:58:00Z")
        ),
    );
    std::fs::write(
        world.path("adapters/adapter.toml"),
        format!(
            "[adapter.releases]\ncommand = [\"{}\"]\nconfig = \"config.schema.json\"\nsecrets = []\n",
            script.display()
        ),
    )
    .unwrap();
    std::fs::write(world.path("adapters/config.schema.json"), "{}\n").unwrap();
    let mut settings = family_settings("boomdrive-update", &world.path("target-repo"));
    settings.push_str(&format!(
        "\n[adapters.releases]\nhost = \"{}\"\nschedule = \"*/5 * * * *\"\n",
        common::HOST
    ));
    world.seed(&[(".config/millpond.toml", &settings)]);
    let tick = world.json(&["tick"], LANDED_AT, "");
    assert_eq!(tick["ran"][0]["adapter"], "releases", "{tick}");
    assert_eq!(tick["ran"][0]["report"]["complete"], true, "{tick}");
    let status = world.json(&["status"], LANDED_AT, "");
    assert_eq!(status["adapters"]["releases"]["cursor"], "2026-11-19");
    // Not due again until 10:05; then the same release is held (S2).
    let tick = world.json(&["tick"], "2026-11-19T10:04:00Z", "");
    assert_eq!(tick["ran"], serde_json::json!([]));
    let report = world.json(&["run", "releases"], "2026-11-19T10:06:00Z", "");
    assert_eq!(report["held"], 1);
}
