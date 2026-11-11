# Native package coverage observations

Run the same unchanged Rust fixtures through the native compiler and the Rust
oracle independently:

```sh
python3 tools/native-qualification.py \
  --lust /absolute/package/bin/lust \
  --rustc /absolute/package/bin/rustc \
  --output /absolute/new-evidence-directory
```

The output directory must not exist. Every first compiler and program result is
retained, including stdout, stderr, command arguments, exit code, timeout state,
binary/source hashes, and elapsed nanoseconds. The runner never turns a failed
native compilation into success by using the oracle's artifact.

The existing-subset fixture combines generics, a user trait, iteration, and enum
matching. Negative cases check type, move, and exclusive-borrow rejection. The
expansion cases exercise unsafe pointer reads, async/await execution, generic
associated types, a standard iterator trait object, return-position `impl Trait`
in a trait, field offsets, a C-ABI function, and native library emission. The
library case requires a separate rustc consumer to import the actual rlib and
execute its exported function; a C archive or metadata placeholder cannot pass.

The default exit status qualifies this bounded preview test: the oracle must
validate every case, native existing behavior and rejection must pass, and
unsupported native expansion cases must fail cleanly without an artifact.
Timeouts, compiler crashes, unusable invocations, and accepted runtime or
artifact mismatches fail the gate. `report.json` separately lists the native
coverage gaps; a successful preview gate does not hide them.

Use `--require-all-native` when every listed native expansion must pass. Even
that result would qualify only this fixture set. The report always retains
`full_native_coverage: false` and names the unassessed obligations, including
arbitrary Cargo projects, procedural macros, build scripts, cross-crate native
artifact interoperability, warm edit invalidation, and other targets.

The native lane disables the daemon and uses a fresh cache directory per case.
Each command has one recorded observation. These timings identify observed
work and failures; they do not establish a speedup or preservation of Lust's
performance. Package performance qualification is separate.
