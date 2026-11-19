# keeper-policy Specification

## Purpose
The keeper's policy as captures land (A.14): each capture's own signal
read by the policy in the write that brings it, a matched signal joined
into its rule's current item by the facts its offer stated, and a refused
policy running no rule (125 to 130).

## Requirements

### Requirement: The policy is read in each capture's write

Where the settings' policy stands and holds rules, each capture SHALL be
taken with it, so a rule that matches the capture's stated facts moves its
own signal in the same write as the capture, the move written as made by
`rule:<name>`; a signal no rule matches stays unmoved (127, 128, 129).

#### Scenario: The family's rule gathers a release

- **WHEN** the rule `family` matches the fact `identity` containing `pkg:github/boomdrivedev/` and the release `release/github/boomdrivedev/sluice/0.97.0` lands
- **THEN** the inbox's moves record the move by `rule:family`, and the item lists the signal `release/github/boomdrivedev/sluice/0.97.0#1`

### Requirement: A rule's item over its window

A rule SHALL gather what it matches into its current item, a proposed item
it made whose window has not closed, named for the rule and the window's
close; once that item is closed, the rule's next match SHALL make its next
item (130).

#### Scenario: The window of ten minutes

- **WHEN** the rule `family` has the window `*/10 * * * *` and a release lands at 10:03
- **THEN** it joins the proposed item `family-20261119T101000Z`, of priority `normal`

#### Scenario: A match after the close

- **WHEN** that item closed at 10:10 and timber's release lands at 10:13
- **THEN** it joins a new item, `family-20261119T102000Z`

### Requirement: A refused policy runs no rule

Where the policy is refused, millpond SHALL run no rule, every signal
SHALL stay unmoved for curation, and the inbox's status and each tick
SHALL say why the policy is refused (125).

#### Scenario: A level with no number

- **WHEN** the policy is refused and a release lands
- **THEN** the status names the refusal, counts one unmoved signal of source `release`, and lists no item
