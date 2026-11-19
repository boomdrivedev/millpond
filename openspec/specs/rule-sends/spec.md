# rule-sends Specification

## Purpose
What the tick on the policy's host does with a rule's items: closes each
whose window has come with its brief, sends each closed item of a rule
naming a place kind to a place of that kind through `flume launch`, once,
and writes the place's landing once, read from sluice's record (A.9;
52, 55, 115, 117, 130, 131; C.5, C.7).

## Requirements

### Requirement: Only the policy's host closes and sends

Only a tick on the host `[policy] host` names SHALL close or send a rule's
items; a tick on any other host SHALL run its adapters and read landings
alone (3, 130).

#### Scenario: A tick on another host

- **WHEN** a tick runs on a host the policy does not name
- **THEN** it closes and sends nothing

### Requirement: A rule's item closes at its window with its brief

The policy's host's tick SHALL close each open item of a rule whose window
has come since the item was made, or at once for a rule with no window,
by weir's close, which writes the item's brief of its signals' excerpts in
the order of their event times (130, 131; weir 129).

#### Scenario: A window not yet closed

- **WHEN** the item of a ten-minute window made at 10:03 is ticked at 10:05
- **THEN** nothing is closed and nothing is sent

#### Scenario: The first tick after the window

- **WHEN** the same item, holding sluice 0.97.0's and flume 0.30.0's signals, is ticked at 10:10:30
- **THEN** it is closed, and its brief is sluice's notes then flume's

### Requirement: The send through flume launch

A closed item with a brief and no send, of a rule that names a place kind
and a repository and whose level has a number, SHALL be sent by `flume
launch --json` with `--kind` the rule's kind, `--name <item>--1`, and
`--repository` the rule's repository, `~/` read as the home folder (52;
C.5).

#### Scenario: The family's item launched

- **WHEN** the item `family-20261119T101000Z` of the rule `family`, of kind `boomdrive-update`, closes
- **THEN** flume is asked to launch `boomdrive-update` named `family-20261119T101000Z--1` in the rule's repository

### Requirement: What the launch carries

The launch SHALL carry the item's brief by `--brief-file`, each signal's
stated facts in event-time order by `--params-file`, the number the
settings give the rule's level by `--priority`, and by `--mark` the inbox,
the item, the send's number and `rule:<name>` (51, 131; C.5).

#### Scenario: The parameters and the mark

- **WHEN** the item `family-20261119T101000Z` holds sluice 0.97.0's signal and then flume 0.30.0's
- **THEN** the parameters file lists sluice's facts, identity, version `0.97.0` and pin `0.95.1`, then flume's, the priority is 20, the number `[priorities]` gives `normal`, and the mark names the item and `rule:family`

### Requirement: One send per item, recorded after the launch

A send SHALL be recorded on the item, as the target
`place:<kind>:<name>:<repository>`, only after flume launched the place;
a launch flume refuses SHALL be named in the tick's errors and record
nothing; and an item that holds a send SHALL never be sent again (53,
115).

#### Scenario: A second tick

- **WHEN** the item was sent and the next tick runs
- **THEN** nothing is sent, flume was called once, and the item is listed as sent with its send open

### Requirement: The landing read from sluice's record, once

Each tick SHALL read, for each open send to a place, `sluice status <base>
--all --json` in the send's repository, `<base>` its default branch, and
where a landed submission's head ends with `/<place name>`, write the
send's landing, naming the branch it landed onto and the revision, once
(55, 117; C.7).

#### Scenario: The place's work lands

- **WHEN** sluice's record shows `update/family-20261119T101000Z--1` landed on main as `abc1234`
- **THEN** the tick writes the landing, the item's send shows `landing main abc1234`, and the next tick writes none

### Requirement: An inbox whose items go to a planner sends nothing

Where the settings say items go to a planner, the policy's host's tick
SHALL close a rule's items and SHALL send none to a place (50, 132).

#### Scenario: Items to the planner

- **WHEN** the settings give `items = "planner"` and a rule's item closes
- **THEN** the tick sends nothing
