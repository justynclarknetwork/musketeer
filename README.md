# Musketeer

[![CI](https://github.com/justynclarknetwork/musketeer/actions/workflows/ci.yml/badge.svg)](https://github.com/justynclarknetwork/musketeer/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/justynclarknetwork/musketeer)](https://github.com/justynclarknetwork/musketeer/releases/tag/v0.3.0)
[![Rust](https://img.shields.io/badge/rust-edition%202021-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

Musketeer is the public trio topology pack for SMALL-governed workspaces.

SMALL defines canonical execution state. Musketeer packages role-separated originator, examiner, and executor workflows against that state, adding packets, verdicts, and execution receipts without redefining the base contract.

## Architecture

Musketeer operates on a two-namespace workspace model:

### `.small/` - Canonical state (owned by SMALL)

Contains the protocol-defined execution artifacts:

- `workspace.small.yml` - workspace identity and metadata
- `intent.small.yml` - what you want to accomplish
- `constraints.small.yml` - boundaries and requirements
- `plan.small.yml` - tasks to be performed
- `progress.small.yml` - execution progress
- `handoff.small.yml` - structured handoff between roles

Musketeer reads from `.small/` but never writes to it. These artifacts are owned by the SMALL protocol.

### `.musketeer/` - Execution layer (owned by Musketeer)

Contains Musketeer-specific execution state:

- `musketeer.yml` - workspace configuration
- `packets/` - role context packets
- `verdicts/<replayId>.verdict.yml` - auditor verdict records
- `runs/<replayId>/execution-log.yml` - execution logs

### Canonical file tree

```
.small/
  workspace.small.yml
  intent.small.yml
  constraints.small.yml
  plan.small.yml
  progress.small.yml
  handoff.small.yml

.musketeer/
  musketeer.yml
  packets/
  verdicts/
    <replayId>.verdict.yml
  runs/
    <replayId>/
      execution-log.yml
```

## Quickstart

```sh
cargo build
```

Initialize a workspace (bootstraps `.small/` if missing, creates `.musketeer/`):

```sh
musketeer init
```

Create a run, generate packets, log progress, record a verdict, and validate:

```sh
musketeer run new
musketeer packet --role planner
musketeer log --role executor --kind note --message "completed task 1"
musketeer verdict --role auditor --value approve --reason "all checks passed"
musketeer check
```

## Migration

Legacy workspaces (artifacts under `.musketeer/runs/`) can be converted to SMALL-native layout:

```sh
musketeer migrate            # convert in place
musketeer migrate --dry-run  # preview without changes
```

SMALL-native is the canonical model going forward. Legacy shadow artifacts are deprecated.

## Commands

| Command | Description |
|---------|-------------|
| `init` | Bootstrap `.small/` if missing, create `.musketeer/` |
| `run new` | Create a new execution run in `.musketeer/runs/` |
| `run status` | Show run state (workspace-mode-aware) |
| `check` | Validate SMALL state and Musketeer execution state |
| `packet` | Read from `.small/`, generate role context packets |
| `log` | Read SMALL progress, write execution-log.yml in `.musketeer/runs/` |
| `verdict` | Write auditor verdict to `.musketeer/verdicts/` |
| `migrate` | Convert legacy workspaces to SMALL-native layout |

All commands support `--json` for machine-readable output and `--replay <id>` where applicable.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.85+

## Development

```sh
cargo fmt          # format code
cargo test         # run all tests
```

## License

MIT (see `LICENSE`).
