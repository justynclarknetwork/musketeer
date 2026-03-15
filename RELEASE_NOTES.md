# Release Notes - v0.2.0

## What this release guarantees

- `musketeer init` initializes a workspace (`.musketeer/`) and is safe to re-run.
- `musketeer run new` creates a run with a UUID replay_id and all required YAML files.
- `musketeer run status` reads and summarizes run state from disk.
- `musketeer check` validates all run invariants: file presence, replay_id consistency, sequence integrity, plan task uniqueness. Exits 0 on pass, non-zero on failure.
- `musketeer packet` returns a structured JSON context packet for a given role (planner, executor, auditor).
- `musketeer log` appends a progress entry with a monotonically increasing sequence number.
- `musketeer verdict` records an auditor approve or reject decision on a run.
- All commands support `--json` for machine-readable output with a stable shape: `{"tool","version","status","replay_id","errors",[...command fields]}`.
- Exit codes are stable and documented: 0 (success), 20 (invariant failed), 21 (role violation), 22 (handoff invalid), 23 (verdict rejected), 30 (workspace/run missing), 40 (invalid input), 50 (internal error).
- Error output in `--json` mode always includes an error code string.
- Workspace state is written atomically (temp file + fsync + rename).

## What is still intentionally limited

- Run YAML files (`intent.yml`, `constraints.yml`, `plan.yml`, etc.) are created with empty/default values. Editing them requires direct file modification; no CLI commands exist for updating intent or constraints.
- `--replay` defaults to the lexicographically last run ID when omitted. There is no interactive run selection.
- `packet --max-bytes` flag is accepted but not enforced.
- No interactive TUI mode is exposed in this release despite `ratatui` being present in dependencies.
- No bridge integration is bundled in the CLI binary. Bridge is a separate daemon.
- Redaction policy config exists in `musketeer.yml` but is not enforced.

## Test coverage

- Integration tests: `tests/cli_spine.rs` (6 library-level tests)
- Contract/smoke tests: `tests/contract_cli.rs` (full workflow + per-command invalid-path tests for all 6 commands)
- All tests use isolated temp workspaces.

## Verified behavior

The following was run against the v0.2.0 binary and passed:

```
cargo test
```

All tests pass. See DEMO.md for exact replay commands.
