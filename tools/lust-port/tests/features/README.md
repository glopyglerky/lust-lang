# Feature probes

One small program per feature id in `../../features.json`. A second
program with a `_b` suffix probes a second form of the same feature.
Files with `// helper: true` are module targets for another probe.

Header lines the runner reads:

- `// feature: <id>`: the feature id this program probes.
- `// form: <text>`: the exact form, in ts-rust style.
- `// rustc-panic: unwind`: compile the rustc oracle with unwinding.
- `// oracle: lust-only`: no single-file rustc oracle exists (external
  crate or proc macro). The runner records the lust result only.

Run `python3 scripts/port-scan-sync.py` from the repo root. It builds
each probe with rustc 1.99 (edition 2024) and with `lust check` plus
`lust run`, then reports one status per feature: `supported` (all
probes match rustc), `partial` (some probes match), or `unsupported`.
With `--write` it updates `status` and `milestone` in `features.json`.
