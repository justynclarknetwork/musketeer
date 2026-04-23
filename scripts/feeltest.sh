#!/usr/bin/env bash
set -euo pipefail

fail() {
  echo "MUSKETEER_FEELTEST_FAIL: $1" >&2
  exit 1
}

check_clean_stderr() {
  local file="$1"
  if grep -qi "deprecated" "$file"; then
    fail "unexpected deprecation noise in $(basename "$file")"
  fi
}

command -v jq >/dev/null 2>&1 || fail "jq is required"
command -v musketeer >/dev/null 2>&1 || fail "musketeer is required in PATH"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

musketeer init --json > init.json 2> init.err
check_clean_stderr init.err
jq -e '.tool == "musketeer" and (.errors|type=="array") and .mode == "small_native"' init.json >/dev/null || fail "init json invalid"

musketeer run new --json > run_new.json 2> run_new.err
check_clean_stderr run_new.err
replay_id="$(jq -r '.replay_id' run_new.json)"
[ -n "$replay_id" ] && [ "$replay_id" != "null" ] || fail "missing replay_id"
jq -e '.mode == "small_native"' run_new.json >/dev/null || fail "run_new not small_native"

musketeer packet --role planner --replay "$replay_id" --json > packet.json 2> packet.err
check_clean_stderr packet.err
jq -e '.role == "planner" and (.errors|type=="array")' packet.json >/dev/null || fail "packet invalid"

musketeer log --role executor --kind note --message "x" --replay "$replay_id" --json > log.json 2> log.err
check_clean_stderr log.err
jq -e '.seq == 1 and (.errors|type=="array")' log.json >/dev/null || fail "log invalid"

musketeer verdict --role auditor --value reject --reason "x" --replay "$replay_id" --json > verdict_reject.json 2> verdict_reject.err
check_clean_stderr verdict_reject.err
set +e
musketeer check --replay "$replay_id" --json > check_reject.json 2> check_reject.err
code=$?
set -e
check_clean_stderr check_reject.err
[ "$code" -eq 23 ] || fail "check after reject expected 23 got $code"
jq -e '.errors|index("E_VERDICT_REJECTED") != null' check_reject.json >/dev/null || fail "missing reject code"

musketeer verdict --role auditor --value approve --reason "resolved" --replay "$replay_id" --json > verdict_approve.json 2> verdict_approve.err
check_clean_stderr verdict_approve.err
musketeer check --replay "$replay_id" --json > check_ok.json 2> check_ok.err
check_clean_stderr check_ok.err
jq -e '.status == "ok"' check_ok.json >/dev/null || fail "check after approve invalid"

echo "MUSKETEER_FEELTEST_OK"
