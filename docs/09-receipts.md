# Receipts

A receipt is narrow closure, not a story.

`logline-lab receipt prepare --file receipt.json` performs three gates:

1. full LogLine Act schema validation;
2. receipt-candidate schema validation;
3. evidence reference enforcement.

A candidate must declare:

- `claim`
- `scope_closed`
- `scope_not_closed`
- `evidence_refs`
- `ghosts_remaining`

The database repeats the evidence rule with a check constraint so clients cannot bypass the CLI.
