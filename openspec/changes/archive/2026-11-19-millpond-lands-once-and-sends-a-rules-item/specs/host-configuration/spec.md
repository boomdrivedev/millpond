# Spec Delta

## Purpose

The host's configuration and directories (C.3): which inboxes a host
keeps, the adapters' declarations it can run, where its secrets lie, its
name, and where it keeps each inbox's raw bank and per-host state.

## ADDED Requirements

### Requirement: The host's configuration file

millpond SHALL read the host's configuration from the file `MILLPOND_CONFIG`
names, else `$XDG_CONFIG_HOME/millpond/config.toml`, a missing file being
an empty configuration: its `host` name, else the machine's short
hostname; its `secrets` folder; its adapters' `declarations`; and each
`[[inbox]]` with its `name`, `repository` and `branch`, `inbox` by
default (C.3).

#### Scenario: No configured name

- **WHEN** the configuration names no `host`
- **THEN** the machine's short hostname names the host in pointers, settings and writes

### Requirement: Which inbox a command reaches

A command SHALL reach the inbox `--inbox <name>` names among the
configured ones, the only one where none is named, or, with `--branch`,
the repository `--inbox` names at that branch; it SHALL refuse, saying
how to name one, where the host keeps none or several and none is named
(C.1, C.3).

#### Scenario: Several inboxes and none named

- **WHEN** the host keeps two inboxes and a command names neither
- **THEN** it is refused, saying to name one with `--inbox <name>`

### Requirement: Local and remote inboxes

An inbox whose repository is a path on the host SHALL be read and written
in that repository's git directory; any other SHALL be reached as a
remote through a bare cache under `$XDG_CACHE_HOME/millpond/git/` (C.2,
C.3).

#### Scenario: A remote inbox

- **WHEN** an inbox's repository is an address and not a path on this host
- **THEN** millpond reaches its branch through the bare cache named for the inbox

### Requirement: Each inbox's host directories

Each inbox SHALL have its raw bank under `$XDG_DATA_HOME/millpond/<inbox>`
and its per-host state under `$XDG_STATE_HOME/millpond/<inbox>`, each
defaulting under `$HOME` as XDG does (C.3).

#### Scenario: The family inbox's bank

- **WHEN** the inbox `family` banks a capture on a host with `XDG_DATA_HOME` set
- **THEN** its raw files are under `$XDG_DATA_HOME/millpond/family/<key>/`
