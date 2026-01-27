# Musketeer state layout

The CLI stores local state in `.musketeer/` under the workspace root.

## Layout

```
.musketeer/
  musketeer.yml
  runs/
    <replay_id>/
      intent.yml
      constraints.yml
      plan.yml
      progress.yml
      handoff.yml
```

Runs are stored under `.musketeer/runs/` using their `replay_id` as the directory name.

## Latest run selection

When a command needs a default run (for example `musketeer check` without `--replay`),
it picks the lexicographically last run id to keep behavior deterministic.
