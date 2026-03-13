# Musketeer

Musketeer is a governed execution harness for role-separated AI work.

It structures planning, challenge, execution, and review into explicit stages with clear handoffs, bounded loops, and auditable outcomes.

Musketeer does not replace models, agents, or editors. It governs how work moves through them.

---

## Core Idea

Musketeer operates on a strict three-role model:

1. **Originator**
   - Intent formation
   - Scope and constraint definition
   - Handoff preparation

2. **Examiner**
   - Adversarial validation
   - Assumption testing
   - Drift detection

3. **Executor**
   - Bounded execution
   - Artifact production
   - Results for review

Each role is isolated.
Each handoff is explicit.
All state lives on disk.

---

## What Musketeer Is

- A CLI
- Local-first
- File-based state
- Role-driven execution governance
- Model-agnostic via adapters
- Built for replay, inspection, and audit

---

## What Musketeer Is Not

- Not an agent framework
- Not a chat wrapper
- Not an orchestration SDK
- Not a hosted service
- Not autonomous

Musketeer does not try to make models smarter. It makes model-driven work more governable.

---

## Design Principles

- Explicit handoffs over implicit memory
- Role separation over model selection
- Bounded execution over open-ended generation
- Contracts over conventions
- Determinism over creativity

---

## Status

This repository is under active construction. No public releases yet.

---

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2021, toolchain 1.85+)

## Getting Started

Build the CLI:

```sh
cargo build
```

### Workflow

1. **Initialize a workspace** in the current directory. This creates a `.musketeer/` directory with a config file and a `runs/` folder:

   ```sh
   cargo run -- init
   ```

2. **Create a new run.** Each run gets a unique UUID and five YAML state files (intent, constraints, plan, progress, handoff):

   ```sh
   cargo run -- run new
   ```

3. **Check run status.** Shows task progress for all runs, or a specific run with `--replay <id>`:

   ```sh
   cargo run -- run status
   cargo run -- run status --replay <replay-id>
   ```

4. **Validate invariants.** Checks file presence, replay ID consistency, progress sequence integrity, and plan task uniqueness. Validates the latest run by default, or a specific one with `--replay <id>`:

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
