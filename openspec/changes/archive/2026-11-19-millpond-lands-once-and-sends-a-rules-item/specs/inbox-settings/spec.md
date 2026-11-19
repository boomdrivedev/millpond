# Spec Delta

## Purpose

An inbox's settings, `.config/millpond.toml` at the root of its branch,
read at the version the inbox was read at, never written by millpond: the
limit, where items go, the lock's staleness, the adapters, and the
keeper's policy with the checks C.2 gives it (2, 125; C.2).

## ADDED Requirements

### Requirement: The settings and their defaults

The settings SHALL be read from `.config/millpond.toml` at the branch's
tip, with a limit of 8192 bytes, items going to sends and a lock stale
after 3600 seconds where they give none, and SHALL be refused, naming the
field, where a top-level field is not one millpond or weir's policy
defines or a value is not of its kind (2; C.2, C.4).

#### Scenario: An unknown field

- **WHEN** the settings hold a top-level field `colour`
- **THEN** the settings are refused, saying `colour` is no field of the settings

### Requirement: Each adapter names its host and schedule

Each `[adapters.<name>]` table SHALL name the one `host` the adapter runs
on and its `schedule`, and MAY give its `config`; a table missing either
is refused naming it, and a schedule that does not parse leaves that
adapter not run, saying why (3, 28; C.2, C.4).

#### Scenario: An adapter with no host

- **WHEN** `[adapters.releases]` names a schedule and no host
- **THEN** the settings are refused, saying `adapters.releases.host` is missing

### Requirement: The policy is read by weir's crates with C.2's checks

The rules and the people list SHALL be read as weir's crates read them.
The policy SHALL be refused, with why, where a rule names a priority level
`[priorities]` gives no number, where a rule that gathers items has no
`[policy] host` to close them, or where `[policy]` or `[priorities]`
holds what they do not define. A refused policy SHALL leave the rest of
the settings standing (125, 130, 131; C.2).

#### Scenario: A level with no number

- **WHEN** the rule `family` asks for priority `normal` and `[priorities]` gives `normal` no number
- **THEN** the policy is refused naming `rules[0].priority`, and the adapters and limit still stand
