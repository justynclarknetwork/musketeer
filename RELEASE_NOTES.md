# Release Notes - v0.3.0

## SMALL-native migration

Musketeer is now SMALL-native. This is the defining change of v0.3.0.

- `.small/` is the canonical state directory, owned by the SMALL protocol
- `.musketeer/` is the execution namespace, owned by Musketeer
- `musketeer migrate` is the supported path for converting legacy workspaces
- Legacy same-name shadow artifacts (intent.yml, plan.yml, etc. under `.musketeer/runs/`) are deprecated and no longer written

### What changed

- `musketeer init` bootstraps `.small/` if absent and creates `.musketeer/`
- `musketeer run new` creates execution state in `.musketeer/runs/<id>/` only; no artifacts are written to `.small/`
- `musketeer check` validates both SMALL canonical state and Musketeer execution state
- `musketeer packet` reads from `.small/` to generate role context packets
- `musketeer log` writes execution-log.yml in `.musketeer/runs/<id>/`, not legacy progress.yml
- `musketeer verdict` writes to `.musketeer/verdicts/<id>.verdict.yml`
- `musketeer migrate` converts legacy workspaces: archives to `.musketeer/legacy/<timestamp>/`, creates `.small/` with converted artifacts, writes `.musketeer/migration-report.yml`
- Legacy workspace detection emits deprecation warnings on stderr

## Stable guarantees

- JSON contract (`--json`): stable envelope with `tool`, `version`, `status`, `replay_id`, `errors`
- Exit codes: 0 (success), 20 (invariant), 21 (role violation), 22 (handoff invalid), 23 (verdict rejected), 30 (workspace missing), 40 (invalid input), 50 (internal error)
- Atomic writes (temp + fsync + rename)

## Known limitations

- Run YAML files are created with default values; no CLI commands for editing intent or constraints
- `packet --max-bytes` accepted but not enforced
- No interactive TUI mode
- Bridge is a separate daemon, not bundled in CLI

## Test coverage

- 74 tests across unit, integration, and contract suites
- Phase 3 (SMALL-native reads) and Phase 4 (write convergence) test suites verify no legacy artifact leakage
