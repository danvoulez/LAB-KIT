# LogLine Lab Kit

Rust-native operational kit for instantiating a real LogLine Lab.

It loads canon references, creates a Supabase/Postgres act spine, emits nine-slot LogLine Acts, projects state, preserves ghosts, captures evidence, prepares receipts, runs a lab clock, defines hooks, prepares dispatch packets, bounds Hermes workorders, and exposes optional `labd` HTTP surfaces.

```text
LogLine Foundation is the grammar.
LogLine Lab Kit is the operational kit.
A Lab instance declares local practice.
Supabase stores the act spine.
CLI/labd operate the Lab.
Frontends are optional surfaces.
```

## Install

```bash
cargo build
cp .env.example .env
logline-lab init --manifest manifests/santo-andre.manifest.example.yaml
logline-lab doctor
logline-lab status
```

Commands that write semantic state require `SUPABASE_DB_URL` or `--database-url` and write only to `ops.logline_acts`.

## CLI surface

```bash
logline-lab init
logline-lab doctor
logline-lab status
logline-lab canon status
logline-lab act emit --file act.json
logline-lab act get --id <uuid>
logline-lab ghost list
logline-lab ghost open --key ... --what-missing ...
logline-lab evidence add --file evidence-act.json
logline-lab receipt prepare --file receipt-act.json
logline-lab report generate daily-expedition
logline-lab projector run --name all
logline-lab hook run --hook before_receipt_review.require_evidence_refs.v1
logline-lab clock tick --kind daily
logline-lab dispatch prepare --process lab.health.check.v0
logline-lab workorder prepare --dispatch-file dispatch.yaml
logline-lab labd --bind 127.0.0.1:8787
```

## Non-negotiable spine rule

Semantic writes go to `ops.logline_acts`. Projection tables/views are derived surfaces and are never treated as truth.
