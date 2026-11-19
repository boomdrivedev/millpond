# Tasks

This change records millpond v0.1.0 as it landed on main at 8c03517
(#5). Every task below was done by the commit it names, before the
repository kept OpenSpec; each is ticked against that commit, and its
check is the moon task named, which the gate ran at the landing. Every
command runs inside `devenv shell`.

## 1. The toolchain and the flake

- [x] 1.1 Rust at weir's pin in `.prototools`, recutils for weir's write protocol, rustfmt and taplo in treefmt, and a flake exporting the package, its binary naming its revision (5f1d3dc). Verify with `moon run root:format root:hooks`.

## 2. The intake

- [x] 2.1 `millpond land` and `--import` by weir's steps, each capture in a write of its own, the raw bank on the host (intake spec; 080b5a2). Verify with `moon run millpond:test`: `millpond_passes_every_host_case_of_the_pack`, `a_release_lands_once_and_its_rule_gathers_it`.

## 3. The settings and the host

- [x] 3.1 `.config/millpond.toml` read at the tip with C.2's checks of `[priorities]` and `[policy] host` (inbox-settings spec; 080b5a2). Verify with `moon run millpond:test`: `a_level_with_no_number_refuses_the_policy`.
- [x] 3.2 The host's configuration, the inbox a command reaches and the host's directories (host-configuration spec; 080b5a2). Verify with `moon run millpond:test`: every test of `tests/flow.rs` reaches its inbox through a configured host.

## 4. Runs, cursors and status

- [x] 4.1 A run's inputs, refusals, lock, cursor and status per host, and the tick's due adapters (adapter-runs spec; 080b5a2). Verify with `moon run millpond:test`: `a_configured_adapter_runs_and_keeps_its_cursor` and the pack's run cases.

## 5. The policy and the send

- [x] 5.1 The keeper's policy read in each capture's write, gathering into a rule's item over its window (keeper-policy spec; 080b5a2). Verify with `moon run millpond:test`: `a_release_lands_once_and_its_rule_gathers_it`, `a_level_with_no_number_refuses_the_policy`.
- [x] 5.2 The policy's host's tick closes each due item with its brief, sends it through `flume launch` once, and writes the landing from sluice's record once (rule-sends spec; 080b5a2). Verify with `moon run millpond:test`: `a_window_not_closed_sends_nothing`, `the_first_tick_after_the_window_launches_the_rules_kind`, `a_tick_records_the_places_landing_from_sluice`.

## 6. The command line

- [x] 6.1 `land`, `run`, `tick`, `status` and `items`, each answering in JSON (command-line spec; 080b5a2). Verify with `moon run millpond:test`: the tests of `tests/flow.rs`, which read each command's JSON.

## 7. The record

- [x] 7.1 OpenSpec in the repository, with the skills and commands the family's pinned OpenSpec writes under `.claude/` and `openspec/config.yaml` under treefmt's yamlfmt, and this change validated with `openspec validate --strict` and archived, so `openspec/specs/` holds the seven capabilities. Verify with `moon run root:format`, and `openspec validate --specs --strict --no-interactive` by hand: no moon task runs it yet.
