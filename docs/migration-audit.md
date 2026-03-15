# Migration Audit: Musketeer SMALL-Native Convergence

Generated: 2026-03-15
Scope: musketeer (CLI), musketeer-bridge, musketeer.dev (docs site)

---

## Summary

Musketeer currently defines its own intent, constraints, plan, progress, and handoff artifacts with Musketeer-specific schemas, stored under `.musketeer/runs/<replayId>/`. These overlap directly with SMALL's canonical artifact set. Every read, write, model struct, CLI command, test, and doc page assumes Musketeer owns these primitives.

The overlap is total. There is no partial convergence - it is all legacy.

---

## 1. Files where legacy artifacts are READ

### musketeer (Rust CLI)

| File | Artifacts read | Notes |
|---|---|---|
| `src/commands/check.rs` | handoff | Reads handoff to check verdict |
| `src/commands/packet.rs` | intent, constraints, plan, progress | Assembles role packet from all four |
| `src/commands/log.rs` | progress | Reads progress to append entry |
| `src/commands/verdict.rs` | handoff | Reads handoff to set verdict |
| `src/commands/run_status.rs` | (via layout paths) | Reads run dirs |
| `src/invariants/check.rs` | intent, constraints, plan, progress, handoff | Validates all five artifacts |
| `src/fs/layout.rs` | (path definitions) | Defines paths for all five under `.musketeer/runs/` |

### musketeer-bridge (Go)

| File | Artifacts read | Notes |
|---|---|---|
| `internal/runner/runner.go` | none directly | Delegates to CLI via argv; inherits CLI's artifact assumptions |

---

## 2. Files where legacy artifacts are WRITTEN

### musketeer (Rust CLI)

| File | Artifacts written | Notes |
|---|---|---|
| `src/commands/run_new.rs` | intent, constraints, plan, progress, handoff | Creates all five with Musketeer schemas |
| `src/commands/log.rs` | progress | Appends entry and writes back |
| `src/commands/verdict.rs` | handoff | Writes verdict into handoff |

---

## 3. CLI commands touching legacy artifacts

| Command | Reads | Writes | Impact |
|---|---|---|---|
| `musketeer init` | none | config only | Clean - creates `.musketeer/` and config |
| `musketeer run new` | none | intent, constraints, plan, progress, handoff | Creates all five with Musketeer schema |
| `musketeer run status` | run dirs | none | Reads run directory listing |
| `musketeer check` | intent, constraints, plan, progress, handoff | none | Validates all five |
| `musketeer packet` | intent, constraints, plan, progress | none | Reads all four to build packet |
| `musketeer log` | progress | progress | Read-modify-write |
| `musketeer verdict` | handoff | handoff | Read-modify-write |

---

## 4. Model structs (Musketeer-owned schemas that shadow SMALL)

| File | Structs | Fields |
|---|---|---|
| `src/model/run.rs` | `Intent` | replay_id, title, outcome |
| `src/model/run.rs` | `Constraints` | replay_id, scope, non_goals, allowlist |
| `src/model/run.rs` | `Plan`, `PlanTask` | replay_id, tasks (id, title, status) |
| `src/model/run.rs` | `Handoff` | replay_id, note, verdict, verdict_reason |
| `src/model/progress.rs` | `ProgressLog`, `ProgressEntry` | replay_id, entries (seq, ts, role, kind, message, summary) |

These are all Musketeer-specific schemas. They share names with SMALL artifacts but use different field shapes.

---

## 5. Tests asserting old schema behavior

| File | Tests | What they assert |
|---|---|---|
| `tests/cli_spine.rs` | `init_creates_state_dir`, `run_new_creates_run_files`, `check_passes_on_fresh_run`, `check_fails_if_missing_file`, `check_fails_if_replay_id_mismatch`, `check_fails_if_progress_seq_not_increasing` | All assert Musketeer-schema artifacts under `.musketeer/runs/` |
| `tests/contract_cli.rs` | `init_creates_workspace_files`, `run_new_creates_artifacts`, `run_status_shows_run`, `check_passes_fresh_run`, `packet_returns_role_and_intent`, `log_appends_entry_with_seq`, `verdict_records_approve`, `json_contract_and_exit_codes` | Full CLI contract tests against Musketeer-owned artifacts |

All tests will need updating during migration. No test currently references `.small/`.

---

## 6. Doc pages claiming independence or optional compatibility

### musketeer.dev (docs site)

| File | Issue |
|---|---|
| `src/lib/content/docs/index.md` | "SMALL compatibility is optional. Musketeer runs standalone." - exact wording from handoff spec's "what to avoid" list |
| `src/lib/content/docs/concepts/what-is-musketeer.md` | Describes Musketeer as owning state: "all state lives in `.musketeer/` as YAML files" |
| `src/lib/content/docs/concepts/state-model.md` | Documents intent.yml, constraints.yml, plan.yml, progress.yml, handoff.yml as Musketeer-owned with Musketeer schemas |
| `src/lib/content/docs/concepts/three-roles.md` | (needs review for framing) |
| `src/lib/content/docs/concepts/what-it-is-not.md` | (needs review for framing) |
| `src/lib/content/docs/usage/init.md` | Documents init creating `.musketeer/` only |
| `src/lib/content/docs/usage/run-new.md` | Documents run creating Musketeer-owned artifacts |
| `src/lib/content/docs/getting-started.md` | End-to-end walkthrough uses Musketeer-only model |
| `src/lib/content/docs/cli-reference.md` | Command reference assumes Musketeer-owned state |

### musketeer CLI repo

| File | Issue |
|---|---|
| `README.md` | "No public releases yet" (outdated); no mention of SMALL; describes Musketeer as owning all state |

### musketeer-bridge repo

| File | Issue |
|---|---|
| `README.md` | "SMALL compatibility is optional" framing implied by "Musketeer governs the work cycle" language |

---

## 7. Bridge endpoints assuming Musketeer-owned control plane

The bridge (`musketeer-bridge`) is a generic tool execution daemon. It does not directly read or write Musketeer artifacts. It executes CLI commands via argv and returns stdout/stderr.

However:
- The bridge's registry examples include `musketeer` tool specs that invoke `musketeer init --json`
- The bridge assumes `.musketeer/` is the only workspace structure
- Bridge docs say "Musketeer governs the work cycle" without mentioning SMALL
- No workspace detection for `.small/` exists in bridge code

Bridge migration is lower risk than CLI migration because the bridge delegates to the CLI. Once the CLI reads from `.small/`, bridge behavior follows.

---

## 8. Replay ID ownership

Currently: `run_new.rs` generates a `Uuid::new_v4()` as `replay_id` and stamps it into all five Musketeer artifacts.

Target: `replay_id` must be sourced from SMALL context. Musketeer must not generate its own when a SMALL workspace exists.

---

## 9. File layout overlap

Current:
```
.musketeer/
  musketeer.yml
  runs/
    <uuid>/
      intent.yml
      constraints.yml
      plan.yml
      progress.yml
      handoff.yml
```

Target:
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
  runs/
  bridge/
```

---

## 10. Severity assessment

| Area | Severity | Reason |
|---|---|---|
| `src/model/run.rs` + `progress.rs` | HIGH | These structs define the shadow schemas |
| `src/commands/run_new.rs` | HIGH | Creates all five shadow artifacts |
| `src/fs/layout.rs` | HIGH | Hardcodes all paths under `.musketeer/runs/` |
| `src/invariants/check.rs` | HIGH | Validates against Musketeer schemas |
| `src/commands/packet.rs` | MEDIUM | Reads all four; needs to read from `.small/` instead |
| `src/commands/log.rs` | MEDIUM | Reads/writes progress; needs SMALL read + Musketeer-namespaced write |
| `src/commands/verdict.rs` | MEDIUM | Currently writes to handoff; should write to `.musketeer/verdicts/` |
| Tests (both files) | HIGH | Every test asserts old layout and schemas |
| Docs site (8 pages) | HIGH | Frames Musketeer as owning state |
| Bridge | LOW | Delegates to CLI; follows automatically |

---

## Recommended new modules (from handoff spec)

```
src/small_workspace.rs      - locate .small/, load canonical artifacts, validate, surface replayId
src/replay_source.rs        - canonical replayId from SMALL, reject conflicting legacy identity
src/musketeer_namespace.rs  - manage .musketeer/ dirs, packet/verdict/run path helpers
src/legacy_workspace.rs     - detect old artifact family, distinguish legacy from current mode
src/migration.rs            - perform migration, generate report, archive legacy artifacts
```

---

## Execution order (from handoff spec)

1. Audit current read/write/doc surface (this document)
2. Declare architecture and deprecate overlap
3. Implement SMALL workspace detection
4. Implement `.musketeer/` namespace
5. Switch read path to `.small/`
6. Switch write path to `.musketeer/`
7. Update bridge
8. Add migration command
9. Rewrite docs/examples
10. Release with explicit migration notes
