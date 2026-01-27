# Run invariants

The `musketeer check` command validates the following invariants for a run:

- Required run files exist (`intent.yml`, `constraints.yml`, `plan.yml`, `progress.yml`, `handoff.yml`).
- The `replay_id` stored in each run file matches the directory name.
- Progress log `seq` values are strictly increasing and start at 1 when entries are present.
- Plan task ids are unique.
