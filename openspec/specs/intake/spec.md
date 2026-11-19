# intake Specification

## Purpose
How millpond takes capture-contract lines into an inbox: each offer by
weir's steps, each capture landed once by its key in a write of its own
on the inbox's shared branch, its raw material banked on the host that
took it, and what was taken, held and refused reported in weir's report
(A.2 to A.5, A.11; C.1 to C.3).

## Requirements

### Requirement: Each capture lands once by its key

millpond SHALL take each offer by weir's capture steps, and an offer
whose key the inbox holds SHALL write nothing, bank nothing and be
counted as held (9, 10, 11; C.1).

#### Scenario: The same release landed twice

- **WHEN** the line offering `release/github/boomdrivedev/sluice/0.97.0` is landed, and landed again a minute later
- **THEN** the first landing takes that key, and the second takes none and counts one held

### Requirement: Each capture is a write of its own

Each capture taken SHALL be written to the inbox's shared branch in one
write of weir's protocol, formed from the inbox read at the branch's tip
and checked there, asked for as `millpond@<host>`, and no run SHALL hold
the inbox from other writers for its length (4, 81, 104; C.2).

#### Scenario: Fourteen offers in one run

- **WHEN** a run offers fourteen captures, one keyed under `moves`
- **THEN** thirteen are taken and the fourteenth is refused naming its line, its key and why (weir's pack, `fourteen-offers-one-under-moves`)

### Requirement: Raw material is banked on the host that took it

A capture's raw files SHALL be banked whole on the host that took it,
under `<data>/millpond/<inbox>/<key>/`, with a cut excerpt's whole text
beside them as `_excerpt`, before the capture is written; millpond SHALL
never delete what a taken capture names (14 to 16, 19, 103; C.3).

#### Scenario: The notes banked

- **WHEN** the release `release/github/boomdrivedev/sluice/0.97.0` is landed from a raw place holding its `notes.md`
- **THEN** the host's bank for the inbox holds that `notes.md` under the capture's key

### Requirement: The intake

`millpond land` SHALL take capture-contract lines on standard input, with
`--raw-dir` naming the folder their raw files are in, and print weir's
report, `weir-report/1`, of what it took, held and refused, keeping no
cursor and no adapter's status (27, 106; C.1).

#### Scenario: Weir's conformance pack as a host

- **WHEN** every host case of weir's conformance pack is run against the binary, as `millpond land --inbox <repo> --branch inbox --limit <bytes>` with each step's flags
- **THEN** each printed report and the branch's tip are what the pack expects

### Requirement: The import

`millpond land --import` SHALL take weir's import lines on standard input
in one write, refusing a line that is no import line by its number and
each capture the import refuses by its line (60 to 63, 119, 120; C.11).

#### Scenario: An import written, and again

- **WHEN** three import lines are imported, the third's key beginning with `moves`, and the same lines are imported again
- **THEN** the first import takes two captures and refuses line 3 with why, and the second takes none and counts two held (weir's pack, `an-import-written-as-read-and-again`)
