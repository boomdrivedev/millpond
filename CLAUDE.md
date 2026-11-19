# millpond

The inbox for events that should change a repository's work: it runs
adapters, lands each capture once by key, banks raw material off git,
serves the tools a curation session works through, and sends an item to
one worktree. Its first working version is built: the intake, runs and
ticks, the keeper's policy and a rule's send to flume, in one Rust crate,
`crates/millpond`, on weir's crates at one revision of weir's main.

The design is in boomdrivedev/blueprints: the capture design,
`design/capture/design.md`, and its plan, `plan.md` beside it, from
which millpond's statement, `design/millpond/requirements.md`, is
written. This repository cites the statement's clauses by number and
never restates them. Where this repository and the design disagree, the
design wins.

## The family

**The family.** This tool works with the others of the Boomdrive
family: sluice (the merge queue), Flume (places and their agents),
Boomdrive (the runner), timber, weir and boom. Each tool's statement is
in boomdrivedev/blueprints, `design/<tool>/requirements.md`. Before
designing or fixing a behavior here, look at how the related tools do
the same thing, in their code and their statements: readiness,
configuration, credentials, retries, what a place is. Where two tools do
one thing, reconcile them into one design with one owner the others call
or cite, and say so in the design. Never fix one tool alone when the
cause is that the tools differ.

## Rules

The rules are in `docs/architecture/`: `core.md` binds wherever a change
is made, and an area page, when one exists, binds the paths its
frontmatter names. The decisions behind them are `docs/adr/`, one MADR
record each, proposed until the operator's yes. The founding records and
`core.md` are written from the statement (plan step P2), and the import
below brings `core.md` in once it is written. A design or a commit that
relies on a rule cites its id; one that departs from a rule says which
and why. Where `docs/architecture/` and the narrative docs disagree,
`docs/architecture/` wins. A rule is sent to `core.md` or its area page
with an id, the next free number in its area, amended in the same commit
as the decision that changes it.

@docs/architecture/core.md

## Checks

Every tool runs through the shell, one command at a time:
`devenv shell -- <command>`. moon runs the checks, tagged by layer
(`.moon/workspace.yml`); `.config/sluice.toml` says which layers each
branch requires:

| layer | what | who requires it |
|---|---|---|
| `check-fast` | the root's format (treefmt), the README's prose (Vale) and a new place's first commit through the hooks; the crate's format, lint, typecheck and tests, weir's conformance pack among them; the site's prose (Vale) and its build | every bolt and main |

main and every bolt take changes only through sluice's merge queue,
which lands a submission once the layers its branch requires pass on
the combination; nothing is pushed to either by hand.

Run suites through sluice, so the agents on this host take turns under
its bound: once your work is committed, `sluice exec main` for the
gate's checks, and while iterating `sluice exec main -- moon run
<project>:<task>`. The projects are those `.moon/workspace.yml` lists:
`millpond` (`crates/millpond`), `site` (`site/`, which
`.github/workflows/pages.yml` publishes) and
`root`.
`devenv shell -- check-fast` runs the fast layer
(`moon run :#check-fast`). The site's own notes are `site/README.md`.

## Commits

Stage by pathspec and commit the paths you wrote: `git add <paths>`, then
`git commit -m "<subject>" -- <paths>`. Never `-a`, never `add .`.
Conventional Commits subjects, scope the part of the tree touched; the
commit-msg hook refuses any other.
