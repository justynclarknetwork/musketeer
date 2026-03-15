# Musketeer

Musketeer is the trio execution harness for SMALL-governed workspaces.

SMALL defines canonical execution state. Musketeer runs role-separated originator, examiner, and executor workflows against that state, adding packets, verdicts, and execution receipts without redefining the base contract.

---

## Architecture

Musketeer operates on a layered workspace model:

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
- `verdicts/` - auditor verdict records
- `runs/` - execution run history
- `bridge/` - bridge execution logs

### Transitional layout

The current release (v0.2.0) uses a legacy layout where all artifacts live under `.musketeer/runs/<uuid>/`. This layout is deprecated. Future versions will require a SMALL workspace with canonical artifacts in `.small/`.

---

## Core Idea

Musketeer operates on a strict three-role model:

1. **Originator** - Intent formation, scope and constraint definition, handoff preparation
2. **Examiner** - Adversarial validation, assumption testing, drift detection
3. **Executor** - Bounded execution, artifact production, results for review

Each role is isolated. Each handoff is explicit. All state lives on disk.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021, toolchain 1.85+)

## Getting Started

Build the CLI:

```sh
cargo build
```

### Workflow

1. **Initialize a workspace:**

   ```sh
   cargo run -- init
   ```

2. **Create a new run:**

   ```sh
   cargo run -- run new
   ```

3. **Check run status:**

   ```sh
   cargo run -- run status
   cargo run -- run status --replay <replay-id>
   ```

4. **Validate invariants:**

   ```sh
   cargo run -- check
   cargo run -- check --replay <replay-id>
   ```

## Development

```sh
cargo fmt          # format code
cargo test         # run all tests
cargo test <name>  # run a single test by name
```

---

## License

MIT (see `LICENSE`).
