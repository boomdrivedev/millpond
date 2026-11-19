## Why

millpond's first working version, v0.1.0, landed on main at 8c03517
(#5), its code in 080b5a2, before the repository kept OpenSpec. This
change records what that version does, requirement by requirement, so
`openspec/specs/` holds millpond's capabilities as built and every later
change amends them. It describes the code at 8c03517 and changes none of
it.

The statement is boomdrivedev/blueprints `design/millpond/requirements.md`,
written from `design/capture/design.md` and its plan. The version is
built on weir's shared crates at weir 63c9959 (weir-capture,
weir-signals, weir-branch, weir-policy, weir-cadence and weir-pack), whose
rules millpond keeps none of its own.

## What Changes

- **The intake lands each capture once.** `millpond land` takes
  capture-contract lines by weir's steps into an inbox on a shared branch,
  each capture in a write of its own, its key held against every capture,
  its raw material banked on the host that took it, and prints
  `weir-report/1`; `--import` takes weir's import lines; weir's
  conformance pack holds millpond as a host (A.2 to A.5, A.11; C.1 to
  C.3).
- **The settings and the host.** `.config/millpond.toml` on the inbox's
  branch gives the limit, where items go, the lock's staleness, the
  adapters with their host and schedule, and the keeper's policy with
  C.2's checks; the host's configuration names its inboxes, declarations,
  secrets and name, and its directories hold each inbox's bank and state
  (C.2, C.3).
- **Adapters' runs, cursors and status per host.** A run is handed its
  configuration, its secrets and no others, an empty raw place and its
  last cursor; it is refused where its configuration or a secret is
  missing, never runs twice at once, moves the cursor only when complete,
  and leaves its status on its host with secrets taken out of its report;
  a tick runs each due adapter of its host (20 to 30, 86 to 88; C.4).
- **The policy.** The keeper's rules read each capture's stated facts in
  the write that lands it, joining a match into the rule's current item
  over its window; a refused policy runs no rule and says why (125 to
  130).
- **The send.** On the policy's host, a tick closes each rule's item at
  its window with its brief, launches each closed item of a rule naming a
  place kind through `flume launch`, one send per item, and writes the
  place's landing once from sluice's record (52, 55, 115, 117, 130, 131;
  C.5, C.7).
- **The command line.** `land`, `run`, `tick`, `status` and `items`, each
  answering in JSON (64, 65, 67).

Out of scope, each a later step of `design/capture/plan.md`:

- Reading, curation's moves and the tools a curation session works
  through (A.6, A.7, A.12), and inquiries and conclusions (A.10, C.8).
- Sends to an inquiry or taken by a planner (50, 132): an inbox whose
  items go to a planner closes its rules' items and sends none.
- The starters (C.6), a place taken over whose send was never recorded
  (116), and the operator closing an open send as abandoned (115).
- millpond's rules in `docs/architecture/` and its founding records,
  plan step P2.

## Capabilities

### New Capabilities

- `intake`: each capture landed once by key, in its own write, its raw
  material banked; the intake and the import.
- `inbox-settings`: `.config/millpond.toml` and the policy's checks.
- `host-configuration`: the host's file, which inbox a command reaches,
  and the host's directories.
- `adapter-runs`: a run's inputs and refusals, its lock, cursor and
  status, and the tick's due adapters.
- `keeper-policy`: rules moving signals by stated facts into a rule's
  item over its window.
- `rule-sends`: closing a rule's item, the send through `flume launch`,
  and the landing from sluice's record.
- `command-line`: the commands and their JSON answers.

### Modified Capabilities

None.

## Impact

Docs only: `openspec/`, the OpenSpec skills and commands under
`.claude/`, CLAUDE.md's Specs paragraph, and `treefmt.nix`, which
formats `openspec/config.yaml` with yamlfmt as the file type's change
must decide. The code it describes is `crates/millpond` and the flake at
8c03517.
