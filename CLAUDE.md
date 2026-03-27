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

Musketeer is the public trio topology pack for governed work. It structures planning, challenge, execution, and review into explicit stages with clear handoffs, bounded loops, and auditable outcomes. All state lives on disk in a `.musketeer/` directory.

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

Each run is a UUID-named directory under `.musketeer/runs/` containing five YAML files: `intent.yml`, `constraints.yml`, `plan.yml`, `progress.yml`, and `handoff.yml`. Every file contains a `replay_id` field that must match its parent directory name.

### Key Design Patterns

- **Atomic writes**: All file writes use a temp file + rename pattern (`write_file_atomic`) to prevent corruption
- **YAML for all state**: Human-readable and editable
- **Invariant checking**: `musketeer check` validates file presence, replay ID consistency, progress sequence integrity (starts at 1, strictly increasing), and plan task ID uniqueness

## Testing

Integration tests live in `tests/cli_spine.rs`. Tests use `tempfile::TempDir` for isolation and a mutex to prevent concurrent interference. The test pattern is: init workspace -> create run -> validate invariants.
