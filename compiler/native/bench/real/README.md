# bench/real

`main.rs` is logscan, a hand-written 1.3k-line log analyzer in the lust
subset: a log parser, a deterministic log generator, per-service latency
percentiles, error-burst detection, and report rendering.

It is the non-cloned benchmark corpus. The other large corpora are one
module cloned many times (`bench/realish`) or generated units
(`bench/rustc-baseline`). This file is varied code, written once.

Rules for changes here:

- rustc and lust must both build it and print the same output.
  `bench/run.py` checks this on every run.
- Keep it in the lust subset (no HashMap, floats, tuples, or string slicing).
- Do not clone parts of it to grow the line count. Add real features instead.
