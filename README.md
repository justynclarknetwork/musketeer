# Musketeer

Musketeer is a local-first, CLI-based execution harness for role-separated AI workflows.

It is designed to make AI-assisted work:
- deterministic
- auditable
- resumable
- governable

Musketeer does not replace models, agents, or editors.
It enforces **how work moves from intent to execution**.

---

## Core Idea

Musketeer operates on a strict three-role model:

1. **Originator**
   - Ideation
   - Planning
   - Intent definition

2. **Cross-Examiner**
   - Adversarial review
   - Constraint enforcement
   - Risk and gap detection

3. **Executor**
   - Deterministic execution
   - Tool usage
   - Artifact production

Each role is isolated.
Each handoff is explicit.
All state lives on disk.

---

## What Musketeer Is

- A CLI
- Local-first
- File-based state
- Role-driven execution
- Model-agnostic via adapters
- Built for replay, inspection, and audit

---

## What Musketeer Is Not

- Not an agent framework
- Not a chatbot
- Not a hosted service
- Not autonomous
- Not magic

Musketeer prioritizes discipline over novelty.

---

## Design Principles

- State over chat history
- Roles over models
- Contracts over conventions
- Explicit handoffs over implicit memory
- Determinism over creativity

---

## Status

This repository is under active construction.

Initial milestones:
- Workspace initialization
- Run and replay identifiers
- On-disk state model
- Invariant enforcement
- Adapter interfaces for external agent CLIs

No stability guarantees yet.
No public releases yet.

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

TBD
