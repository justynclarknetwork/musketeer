#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "MUSKETEER_FEELTEST_FAIL: $1" >&2
  exit 1
}

command -v jq >/dev/null 2>&1 || fail "jq is required"
command -v musketeer >/dev/null 2>&1 || fail "musketeer is required in PATH"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

musketeer init --json > init.json
jq -e '.tool == "musketeer" and (.errors|type=="array")' init.json >/dev/null || fail "init json invalid"

musketeer run new --json > run_new.json
replay_id="$(jq -r '.replay_id' run_new.json)"
[ -n "$replay_id" ] && [ "$replay_id" != "null" ] || fail "missing replay_id"

musketeer packet --role planner --replay "$replay_id" --json > packet.json
jq -e '.role == "planner" and (.errors|type=="array")' packet.json >/dev/null || fail "packet invalid"

musketeer log --role executor --kind note --message "x" --replay "$replay_id" --json > log.json
jq -e '.seq == 1 and (.errors|type=="array")' log.json >/dev/null || fail "log invalid"

musketeer verdict --role auditor --value reject --reason "x" --replay "$replay_id" --json > verdict_reject.json
set +e
musketeer check --replay "$replay_id" --json > check_reject.json
code=$?
set -e
[ "$code" -eq 23 ] || fail "check after reject expected 23 got $code"
jq -e '.errors|index("E_VERDICT_REJECTED") != null' check_reject.json >/dev/null || fail "missing reject code"

musketeer verdict --role auditor --value approve --reason "resolved" --replay "$replay_id" --json > verdict_approve.json
musketeer check --replay "$replay_id" --json > check_ok.json
jq -e '.status == "ok"' check_ok.json >/dev/null || fail "check after approve invalid"

echo "MUSKETEER_FEELTEST_OK"
