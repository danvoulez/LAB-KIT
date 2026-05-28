# CLI

The production command surface is the `logline-lab` binary.

Core commands:

- `init` validates manifests, migrates Supabase/Postgres, and emits seed acts.
- `doctor` checks filesystem assets and live database relations.
- `status` reads projected state from the database.
- `act emit` validates a LogLine Act schema and writes to `ops.logline_acts`.
- `receipt prepare` validates the stricter receipt schema before writing.
- `ghost open`, `evidence add`, `clock tick`, and `report generate` all preserve the act/projection boundary.
- `dispatch prepare` and `workorder prepare` generate governed YAML packets; they do not execute protected consequence.

The CLI is safe to run in CI. Commands that require semantic writes fail unless a database URL is provided.
