# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
cargo build              # Build the project
cargo test               # Run all tests
cargo test <test_name>   # Run a single test by name
cargo fmt                # Format code
cargo run -- init        # Initialize workspace
cargo run -- run new     # Create new run
cargo run -- run status  # Show run status (--replay <id> optional)
cargo run -- check       # Validate run invariants (--replay <id> optional)
```

When no `--replay` is specified, the lexicographically last run ID is selected automatically.

## Architecture

Musketeer is the SMALL-native trio workflow harness CLI. It structures planning, challenge, execution, and review into explicit stages with clear handoffs, bounded loops, and auditable outcomes. Canonical state lives in `.small/`, while Musketeer-owned execution artifacts live in `.musketeer/`.

### Three-Role Model

Every workflow enforces three isolated roles with explicit handoffs between them:
1. **Originator** - intent formation, scope definition, handoff preparation
2. **Examiner** - adversarial validation, assumption testing, drift detection
3. **Executor** - bounded execution, artifact production, results for review

### Module Layout

- **`src/cli.rs`** - Clap-based CLI definition and argument parsing
- **`src/commands/`** - Command implementations (`init`, `run_new`, `run_status`, `check`)
- **`src/model/`** - Data structures for config, runs (intent/constraints/plan/handoff), and progress logs
- **`src/fs/`** - Filesystem operations: path layout (`layout.rs`), YAML read/write, atomic writes, SHA256 hashing
- **`src/invariants/`** - Validation logic for run state (file presence, replay ID consistency, sequence integrity, plan uniqueness)
- **`src/error.rs`** - `MusketeerError` enum using `thiserror`

### Run State Structure

In SMALL-native mode, canonical intent, constraints, plan, progress, and handoff artifacts live under `.small/`. Musketeer-owned run artifacts live under `.musketeer/`, including packets, verdicts, and `runs/<replay_id>/execution-log.yml`.

### Key Design Patterns

- **Atomic writes**: All file writes use a temp file + rename pattern (`write_file_atomic`) to prevent corruption
- **YAML for all state**: Human-readable and editable
- **Invariant checking**: `musketeer check` validates file presence, replay ID consistency, progress sequence integrity (starts at 1, strictly increasing), and plan task ID uniqueness

## Testing

Integration tests live in `tests/cli_spine.rs`. Tests use `tempfile::TempDir` for isolation and a mutex to prevent concurrent interference. The test pattern is: init workspace -> create run -> validate invariants.
