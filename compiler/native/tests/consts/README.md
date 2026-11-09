# Const initializer corpus (ARCH 2.13)

The Wave A1 gate for ARCH 2.13 is an executable corpus over
locally defined, already supported types only. No A1 case may
cite a type from a later wave.

- `pass/*.rs`: A1 cases. Rustc accepts and runs; stdout must
  equal the sibling `.stdout`.
- `fail/*.rs`: rustc rejects. A header line records the
  class: `// rustc-error: E0015`. A stable stderr substring
  is used only when no E code names the case (the generic
  inline-const failure under `later/`).
- `later/pass/*.rs` and `later/fail/*.rs`: the same shapes
  with source-std types, plus the generic inline-const shape.
  Each file's first line tags its stage, for example
  `// stage: W-STD-B` (RefCell), `// stage: W-THREADS`
  (Mutex and OnceLock), or `// stage: W-CHECK-G` (generic
  inline const). Rustc validates these now. A lust gate may
  only cite one when its stage lands, per ARCH 2.13.

Runner: `scripts/test-adapters.sh` runs this corpus with the
layout corpus, because the const evaluator's layout queries
come from the layout engine (ARCH 4.4).
