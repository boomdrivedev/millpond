# Verify: millpond-lands-once-and-sends-a-rules-item

The report on this change, which records millpond v0.1.0 as main holds
it at 8c03517. The change adds no code, so the check is that each
requirement says what the code does, and that the tests holding the code
pass.

## The tests

`cargo test --workspace`, run once at 8c03517 inside `devenv shell` on
2026-11-19: 7 tests passed and none failed or was ignored.

| suite | passed |
|---|---|
| `crates/millpond/tests/flow.rs` | 6 |
| `crates/millpond/tests/pack.rs`, every host case of weir's pack at 63c9959 | 1 |

## Each spec against the code

- `intake`: `take.rs`, `bank.rs`, `store.rs`; the pack's eight host
  cases, among them `the-same-offer-held`, `an-import-written-as-read-and-again`
  and `fourteen-offers-one-under-moves`, and
  `a_release_lands_once_and_its_rule_gathers_it`.
- `inbox-settings`: `settings.rs`; `a_level_with_no_number_refuses_the_policy`.
  The refusals of an unknown field and of an adapter with no host are read
  from the code; no test reaches them.
- `host-configuration`: `host.rs`; every flow test reaches its inbox
  through a configured host. Remote inboxes through the bare cache and
  the refusal of several unnamed inboxes are read from the code.
- `adapter-runs`: `run.rs`, `state.rs`, `tick.rs` (`run_due`);
  `a_configured_adapter_runs_and_keeps_its_cursor` and the pack's
  `a-run-without-its-completion`.
  A missing secret, a run on another host, the lock and the secret taken
  out of a failed run's report are read from the code.
- `keeper-policy`: `take.rs` (`take_with`), `settings.rs`;
  `a_release_lands_once_and_its_rule_gathers_it`,
  `the_first_tick_after_the_window_launches_the_rules_kind` (the next
  item after the close) and `a_level_with_no_number_refuses_the_policy`.
- `rule-sends`: `tick.rs` (`close_due`, `send_closed`, `read_landings`),
  `send.rs`, `items.rs` (`params`); `a_window_not_closed_sends_nothing`,
  `the_first_tick_after_the_window_launches_the_rules_kind`,
  `a_tick_records_the_places_landing_from_sluice`. A tick on another host
  and an inbox whose items go to a planner are read from the code.
- `command-line`: `main.rs`, `items.rs`; the flow tests read each
  command's JSON. The error form and `--version` are read from the code.

## Tasks

Every task of `tasks.md`, 1.1 to 7.1, is ticked against the commit that
did it; 7.1 is this change's own.

## The record

`openspec validate millpond-lands-once-and-sends-a-rules-item --strict`
passed before the archive, and `openspec validate --specs --strict`
after it.
