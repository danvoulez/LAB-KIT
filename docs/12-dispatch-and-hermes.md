# Dispatch and Hermes Boundary

Dispatch packets describe work. They do not execute work.

Hermes workorders are prepared from dispatch context and include:

- mode: `read_only`, `dry_run`, or `apply`;
- target metadata;
- allowed and forbidden actions;
- commands;
- required evidence;
- secret policy.

The kit defaults to safe workorders with `redact_before_store: true` and `secret_values_allowed_in_logs: false`.

No apply mode should be run without authority and an execution window.
