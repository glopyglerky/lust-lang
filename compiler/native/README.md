# Lustr native compiler

This component contains the native subset compiler inherited from Lust and the
changes maintained by Lustr. Its Cargo package is `lustr-native`. The `lust` and
`lust-core` commands and their protocol names remain for compatibility with the
native fixtures and development tools.

This component is separate from the primary coordinator workspace. It needs the
pinned TinyCC library and runtime. Given an installed TinyCC tree:

```sh
LUST_TCC_HOME=/absolute/tcc \
  cargo build --manifest-path compiler/native/Cargo.toml --release --locked
```

Run from the repository root. Set `CARGO_TARGET_DIR` to a build filesystem for
agent builds. The component's existing `scripts/build-tcc.sh` can build its pinned
native dependency; current delivery retains that dependency rather than claiming
an all-Rust compiler. Installation packages keep `bin/lust`, `bin/lust-core`,
`lib/lust/tcc` and `lib/lust/library` together.

The [root README](../../README.md) and [fork design](../../docs/compiler-set/design.md)
define current product behavior. The component's `docs`, `DESIGN.md`, benchmark
corpora and scripts retain native implementation history and qualification tools.
Their historical measurements do not describe arbitrary Rust projects.
