# Spec Delta

## Purpose

Runs of an inbox's adapters on their host: what a run is handed, when it
is refused, how its lines are taken, and the cursor and status it leaves
on its host; and the tick that runs each due adapter (20 to 30, 86 to 88;
C.1, C.3, C.4).

## ADDED Requirements

### Requirement: A run is handed what its settings and host give it

A run SHALL start the command the adapter's declaration names, in the
declaration's folder, with a fresh empty raw place, the cursor its last
complete run kept, its configuration from the inbox's settings in the file
`CAPTURE_CONFIG` names, and as variables the secrets its declaration names
from the host's secrets folder and no others (20, 30; C.1).

#### Scenario: A configured adapter's run

- **WHEN** the adapter `releases`, declared with a command and an empty schema and set to run on this host, is run
- **THEN** its command is started with `--raw-dir` naming an empty folder, and the lines it prints are taken

### Requirement: An adapter lacking its configuration or secrets is not run

An adapter whose configuration does not fit its declaration's schema, or
one of whose secrets the host does not hold, SHALL not be run, and its
status SHALL say why (21).

#### Scenario: A missing secret

- **WHEN** an adapter's declaration names a secret the host's secrets folder does not hold
- **THEN** the adapter is not run, and its status says this host holds no such secret

### Requirement: An adapter runs only on its host and never twice at once

`millpond run <adapter>` SHALL refuse an adapter the settings place on
another host, and a run SHALL hold the adapter's lock in the inbox's
per-host state while it lasts, refusing another run while it is held and
breaking a lock older than the settings' staleness (3, 22).

#### Scenario: An adapter of another host

- **WHEN** `millpond run releases` is asked on a host other than the one `[adapters.releases]` names
- **THEN** it is refused, naming the host the adapter runs on and this host

### Requirement: Only a complete run moves the cursor

A run SHALL be complete only where its adapter ends it with a completion
and exits with success, and only a complete run SHALL move the adapter's
cursor, to the last cursor it offered. A run that is not complete keeps
every capture it took (23, 24, 25).

#### Scenario: A run that offers a cursor

- **WHEN** an adapter's run offers a capture, the cursor `2026-11-19` and a completion, and exits with success
- **THEN** the run is complete and the adapter's cursor on this host is `2026-11-19`

### Requirement: Every run leaves its status on its host

Each run SHALL leave, in one file per adapter in the inbox's per-host
state written whole by replacing it, when it ran, whether it was complete,
why it was not run where it was not, its `weir-report/1`, and for a
failed run the last twenty lines its adapter reported, each secret's value
replaced by its name (26, 75, 86 to 88; C.3).

#### Scenario: A failed run's report

- **WHEN** an adapter holding a secret prints that secret's value on its standard error and exits with failure
- **THEN** its status is not complete, and its last report names the secret in place of its value

### Requirement: A tick runs each due adapter of its host

`millpond tick` SHALL run every adapter of the inbox that names the tick's
host and is due: never run there, or a time its schedule names has come
since its last run, missed times caught up once (28, 29; C.4).

#### Scenario: Due, then not due

- **WHEN** an adapter scheduled every five minutes is ticked at 10:03 having never run, and ticked again at 10:04
- **THEN** the first tick runs it and the second does not, and a later run holds the release the first took
