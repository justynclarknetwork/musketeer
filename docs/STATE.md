# Musketeer State Layout

Last reviewed: 2026-06-17.

Musketeer now uses a two-namespace workspace model:

- `.small/` stores canonical SMALL Protocol state.
- `.musketeer/` stores Musketeer execution-layer state.

Fresh `musketeer init` flows are SMALL-native by default. Legacy shadow state in
`.musketeer/runs/` is supported only as migration input.

## Layout

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
    <replay_id>.verdict.yml
  runs/
    <replay_id>/
      execution-log.yml
```

The `.small/` files are the durable execution source of truth. The
`.musketeer/` files are derived execution receipts, packets, verdicts, and
Musketeer-specific configuration.

## Latest run selection

When a command needs a default run (for example `musketeer check` without `--replay`),
it picks the lexicographically last run id to keep behavior deterministic.

## Evidence status

The current test spine includes the phase 5 system evidence proof. Keep this doc,
the README command table, and `scripts/evidence_report.sh` aligned when changing
the SMALL-native flow, migration behavior, packet generation, verdict storage,
or run evidence layout.
