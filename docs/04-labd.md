# labd

`labd` is the optional HTTP daemon for local or server-side operation.

It exposes the same boundary as the CLI:

- read endpoints for status, daily state, ghosts, evidence, receipts, canon, and runtimes;
- write endpoints that accept LogLine Acts and insert only into `ops.logline_acts`;
- utility endpoints for projectors, clock ticks, dispatch packets, and workorder preparation.

`labd` is not an authority bypass. Protected action still requires an admitted dispatch packet, an authority decision, an execution window, and evidence return.
