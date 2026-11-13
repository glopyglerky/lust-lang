# Trait corpus (Wave A1, W-TRAITS-CORE gate)

Cases for ARCH 2.4 to 2.6, 2.10, 2.12 and the test list in
ARCH 9.3 W-TRAITS-CORE. The runner is `scripts/test-traits.sh`.
It also runs `tests/mgp/`.

Layout:

- `pass/*.rs`: rustc accepts. The program runs and its stdout
  must equal the sibling `.stdout` file.
- `fail/*.rs`: rustc rejects. The first line records the
  rustc error class: `// rustc-error: E0283`. The token after
  the colon must appear in rustc stderr. Use an `E####` code
  when rustc emits one. An optional second line records the
  lust code ARCH assigns: `// lust-expect: L0610`. Lust mode
  does not enforce that code yet.

Each case is one small single-file program. It names the
ARCH rule it pins in its header comment. Cases taken from
review probes keep the probe shape and say so.

Run:

```sh
scripts/test-traits.sh [filter]          # rustc is the oracle
LUST=target/release/lust scripts/test-traits.sh   # report lust gaps, no fail
LUST=... LUST_STRICT=1 scripts/test-traits.sh     # fail on any lust gap
```
