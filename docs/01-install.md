# Install

Build the kit with Rust:

```bash
cargo build --release
```

Prepare environment:

```bash
cp .env.example .env
export SUPABASE_DB_URL=postgres://postgres:postgres@localhost:54322/postgres
```

Initialize a Lab:

```bash
./target/release/logline-lab init --manifest manifests/santo-andre.manifest.example.yaml
./target/release/logline-lab doctor
./target/release/logline-lab status
```

`init` validates the manifest, runs all migrations in lexical order, records migration checksums, emits the Lab instantiation act, and opens the initial review ghost.

Without `SUPABASE_DB_URL`, `init` validates the manifest but refuses to pretend semantic state was written.
