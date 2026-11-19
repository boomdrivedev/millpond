## Context

millpond v0.1.0 was built before this repository kept OpenSpec, from its
statement and the capture design, on weir's crates. This design records
the shape the code took, so later changes amend it from here.

## Goals / Non-Goals

**Goals:**

- Describe what the code at 8c03517 does, each requirement checked
  against the code, its tests through the binary and weir's pack.

**Non-Goals:**

- Any change to the code or the checks.
- The clauses this version does not build (the proposal's out-of-scope
  list).

## Decisions

### D1. weir owns every rule of the contract, the records and the policy

The take (`weir_capture::take`), the inbox's records and writes
(`weir_signals`), the branch's write protocol (`weir_branch`), the policy
(`weir_policy`) and schedules (`weir_cadence`) are weir's. millpond's
crate binds them to a host: `take.rs` writes each capture through
`store::write`, `bank.rs` banks to the host's folder, `settings.rs` adds
C.2's checks around `Policy::read`.

### D2. Every write is formed at the tip

`store::write` hands weir's protocol a closure that forms the write from
the inbox read at the tip, again at each new tip the branch moved to, so
two hosts writing at once never lose a capture (4, 81, 104).

### D3. Per-host state is files, written whole

Each adapter's cursor and status is one JSON file under the inbox's state
folder, replaced whole, with a lock file beside it broken once older than
the settings' staleness (22, 86 to 88).

### D4. The tick does everything a pass owes, in order

`tick::tick`: due adapters first, then each open send's landing, then, on
the policy's host alone, closes and sends. Each step's failures go into
the tick's `errors` and never stop the rest.

### D5. The send launches first and records after

`send_closed` runs `flume launch` with the brief and parameters in files,
and records the send only once flume answered, so no send is recorded for
a place that was not made (53). The place's name is `<item>--1`, since an
item has at most one send to a place in this version (115).

### D6. flume and sluice are programs on PATH

`MILLPOND_FLUME` and `MILLPOND_SLUICE` name the programs in their place,
and `MILLPOND_NOW` fixes the clock, so the tests run the binary against a
fake flume and sluice at fixed times.

## Risks / Trade-offs

- The landing is read from `sluice status <base> --all --json` in the
  rule's repository, which must be a local checkout on the policy's host.
- A rule with no number for its level is refused as a whole policy, so one
  missing number stops every rule until it is fixed.
