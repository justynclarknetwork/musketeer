# Musketeer Demo

End-to-end replay of the canonical workflow. All commands run from a temp workspace.

## Prerequisites

- `musketeer` binary in PATH (or use path to binary directly)
- Optional: `musketeer-bridge` running for tool execution (see bridge README)

## Setup

```sh
mkdir /tmp/msk-demo && cd /tmp/msk-demo
```

## Success path

### 1. init - initialize workspace

```sh
musketeer init --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":null,"errors":[]}
```

Files created:
- `.musketeer/musketeer.yml`
- `.musketeer/runs/`

Re-run is safe and idempotent: exit 0 both times.

### 2. run new - create a run

```sh
musketeer run new --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":"<uuid>","errors":[],"replay_id":"<uuid>"}
```

Note the `replay_id`. Use it in subsequent commands. Files created:
- `.musketeer/runs/<replay_id>/intent.yml`
- `.musketeer/runs/<replay_id>/constraints.yml`
- `.musketeer/runs/<replay_id>/plan.yml`
- `.musketeer/runs/<replay_id>/progress.yml`
- `.musketeer/runs/<replay_id>/handoff.yml`

### 3. check - verify invariants pass on fresh run

```sh
musketeer check --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":"<replay_id>","errors":[]}
```

Exit code: 0

### 4. packet - get role context packet

```sh
musketeer packet --role planner --replay <replay_id> --json
```

Expected output:
```json
{
  "tool": "musketeer",
  "version": "1",
  "status": "ok",
  "replay_id": "<replay_id>",
  "errors": [],
  "role": "planner",
  "intent": {"title": "Untitled", "outcome": "TBD"},
  "constraints": {"replay_id": "<replay_id>", "scope": [], "non_goals": [], "allowlist": []},
  "plan_slice": [],
  "progress_slice": [],
  "next_expected_action": "review_or_close"
}
```

Exit code: 0

### 5. log - append a progress entry

```sh
musketeer log --role executor --kind note --message "task completed" --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":"<replay_id>","errors":[],"seq":1,"kind":"note"}
```

Exit code: 0. The entry is appended to `.musketeer/runs/<replay_id>/progress.yml` with `seq: 1`.

### 6. verdict (approve) - auditor signs off

```sh
musketeer verdict --role auditor --value approve --reason "all tasks verified" --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":"<replay_id>","errors":[],"verdict":"approve"}
```

Exit code: 0

### 7. check after approve - passes

```sh
musketeer check --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"ok","replay_id":"<replay_id>","errors":[]}
```

Exit code: 0

---

## Failure path

### verdict reject followed by check - check fails with exit 23

```sh
musketeer verdict --role auditor --value reject --reason "incomplete" --replay <replay_id> --json
```

Exit code: 0 (verdict recorded successfully)

```sh
musketeer check --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"error","replay_id":"<replay_id>","errors":["E_VERDICT_REJECTED","auditor rejected: incomplete"]}
```

Exit code: 23

### run new without workspace - fails with exit 30

```sh
cd /tmp/empty-dir
musketeer run new --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"error","replay_id":null,"errors":["E_WORKSPACE_INVALID","workspace not initialized: missing /tmp/empty-dir/.musketeer"]}
```

Exit code: 30

### packet with invalid role - fails with exit 21

```sh
musketeer packet --role hacker --replay <replay_id> --json
```

Expected output:
```json
{"tool":"musketeer","version":"1","status":"error","replay_id":null,"errors":["E_ROLE_VIOLATION","role violation: hacker"]}
```

Exit code: 21

### log with invalid kind - fails with exit 40

```sh
musketeer log --role executor --kind todo --message "x" --replay <replay_id> --json
```

Exit code: 40 (`E_INVALID_INPUT`)

---

## Bridge integration

With musketeer-bridge running on `127.0.0.1:18789`:

```sh
# Check bridge health
curl -s http://127.0.0.1:18789/v1/health
# {"exit_code":0,"ok":true}

# Run a tool through the bridge (cwd must be in allowlisted_roots)
curl -s -X POST http://127.0.0.1:18789/v1/tools/musketeer/run \
  -H 'content-type: application/json' \
  -d '{
    "args": {},
    "cwd": "/tmp/msk-demo",
    "env": {},
    "mode": "json",
    "client": {"name": "demo"}
  }'
```

The bridge executes `musketeer init --json` (as defined in the musketeer tool spec), captures stdout, validates it is a single JSON object, logs the run to `~/.musketeer/runs/`, and returns the result.

Expected successful response:
```json
{
  "exit_code": 0,
  "ok": true,
  "stdout_json": {"tool":"musketeer","version":"1","status":"ok","replay_id":null,"errors":[]},
  "stderr": ""
}
```

Failure response when cwd is not allowlisted:
```json
{
  "exit_code": 40,
  "ok": false,
  "error": {"code": "ERR_CWD_NOT_ALLOWLISTED", "message": "cwd is not within an allowlisted root"}
}
```
