# LogLine Lab Kit Overview

The LogLine Lab Kit is a Rust-native operational kit for running a real LogLine laboratory.

It is not a frontend, dashboard, chatbot, or mock local ledger. The kit owns the operational boundary:

1. load canon references;
2. validate Lab manifests;
3. create the Supabase/Postgres semantic spine;
4. emit nine-slot LogLine Acts;
5. expose read models as projections;
6. preserve ghosts;
7. index evidence;
8. prepare receipts only from evidence;
9. run hooks, clock ticks, projectors, dispatch packets, and Hermes workorder boundaries.

The source of semantic truth is `ops.logline_acts`. Every other table, view, endpoint, report, or UI surface is downstream.

## Production invariants

- Semantic writes must use `logline-lab act emit`, `labd POST /acts`, or a server path that writes to `ops.logline_acts`.
- Receipt candidates are rejected unless `this.evidence_refs` is a non-empty array.
- Execution reports are rejected unless `this.secret_redacted` is true.
- Migrations are ordered, checksummed, and recorded in `ops.schema_migrations`.
- Hashes are computed over canonicalized JSON, not incidental object key order.
