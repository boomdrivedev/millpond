# command-line Specification

## Purpose
millpond's command line, one caller of the core printing what a program
reads as JSON (64, 65): `land`, `run`, `tick`, `status` and `items`.

## Requirements

### Requirement: Every answer is JSON

Each command SHALL print one JSON value on success, and on failure SHALL
print `millpond: <why>` on standard error and exit 1 (64, 65).

#### Scenario: A tick's answer

- **WHEN** `millpond tick` runs
- **THEN** it prints one JSON object naming the inbox, the host, the time, the adapters it ran, the landings it wrote, the policy, the items it closed and sent, and its errors

### Requirement: Status asks nothing and writes nothing

`millpond status --json` SHALL give, for this host, each of the inbox's
adapters that names it with its cursor and last status, the policy's
rules and host or why it is refused, and the count and oldest event time
of unmoved signals by source (26, 67).

#### Scenario: An adapter's cursor

- **WHEN** the adapter `releases` completed a run offering the cursor `2026-11-19`
- **THEN** status gives `releases` with the cursor `2026-11-19`

### Requirement: Items are listed as recorded

`millpond items --json` SHALL list each item that holds signals or was
made from a conclusion, with its rule, priority, who named it and when,
when it closed, whether it is proposed or sent, its signals, weight and
brief, and each send with its outcome, open until one is recorded; it
SHALL ask the merge queue nothing (45, 49, 114, 117).

#### Scenario: A proposed item

- **WHEN** the family's rule has gathered one release
- **THEN** items lists `family-20261119T101000Z`, rule `family`, priority `normal`, state `proposed`, with that release's signal

### Requirement: The binary names its revision

`millpond --version` SHALL print `millpond revision <commit>, built by
<rustc>`, the commit the binary was built from.

#### Scenario: Asking the version

- **WHEN** a binary is asked its version
- **THEN** it names the revision it was built from and the compiler that built it
