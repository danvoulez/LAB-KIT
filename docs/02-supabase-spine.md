# Supabase Spine

`ops.logline_acts` is the only semantic write spine.

The table stores the nine LogLine slots plus operational metadata:

- `who`
- `did`
- `this`
- `when`
- `confirmed_by`
- `if_ok`
- `if_doubt`
- `if_not`
- `status`
- `runtime_envelope`
- `tuple_hash`
- `content_hash`
- `previous_act_refs`
- `evidence_state`
- `promotion_state`

Database constraints enforce non-empty identity/action/status, evidence-backed receipts, and redacted execution evidence.

Projection schemas (`audit`, `registry`, `lab_observability`, `evidence`, `receipts`, `workorders`, `authz`) are downstream and reconstructible.
