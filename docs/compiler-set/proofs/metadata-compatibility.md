# Optional Cargo metadata field: comparison contract

Cargo permits new metadata fields within format version 1. This is an
[explicit compatibility guarantee](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html#compatibility),
not evidence that arbitrary new fields can be discarded during a compiler
comparison. A non-null optimization hint can change compilation and must remain
visible.

In the pinned Cargo source
[`31d3246e0e8d3a80d4a90b54aa48ff180adb408b`](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/crates/cargo-util-schemas/src/manifest/mod.rs#L1697),
`Hints.min_opt_level` is an `Option<toml::Value>`. The serializer emits null for
`None`; the earlier tested Cargo omits this field. The pinned
[profile parser](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/src/workspace/profiles.rs#L436)
maps an absent optional value to no optimization floor. The minimum optimization
hint's [documented behavior](https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#package-min-opt-level-hint)
also explains why a non-null value cannot be ignored.

Require Cargo metadata format version 1; missing or other versions are refused.
Let J be a parsed Cargo metadata JSON tree after the experiment's recorded
literal path substitutions. Define N(J) as follows: for each object in the
top-level `packages` array whose `hints` value is an object, remove its
`min-opt-level` key only if that key's value is JSON null. Preserve every other
key/value, array position and scalar value. Missing keys cause no change.
The runner compares N(J) using JSON value equality and records every removal.

Under the stated source/serialization contract:

1. If two trees differ only by absent versus null values at those exact optional
   paths, N maps them to equal trees. Each null key is removed, leaving identical
   objects; all other positions are unchanged.
2. N is idempotent. After one application no removable null key remains, so
   another application makes no change.
3. Every non-null `min-opt-level` and every other field is preserved. Therefore
   a difference in one of those values remains a difference after N, provided
   the earlier path substitutions did not erase it. Unknown null fields,
   `mostly-unused`, features and dependency selections receive no exception.
4. JSON object key order is irrelevant to value equality. Arrays are not sorted
   or projected, so their order and members remain comparison inputs.

These are structural properties of N and a conditional identification of one
unset optional value. They do not prove Rust semantic equivalence, default
profile equivalence across Cargo versions, complete build-input tracing or any
speedup. Actual compiler flags, fingerprints and runtime/rejection gates remain
required. Compiler/Cargo version confounders remain in the report.

Complete raw metadata is retained with its own SHA256 before the comparison.
`metadata_comparison_rule` identifies `cargo-optional-min-opt-level-none-v1`;
`metadata_comparison_projections` lists the lane, package and exact null field
treated as absent. There is no general unknown-field filter or null stripping.

The real-Cargo regression in `crates/lustr-bench/tests/metadata.rs` reproduces
the old refusal with an unchanged tiny project and two Cargo versions using
the same produced Rust compiler. After the change, an unset hint passes every
gate. Adding the actual `min-opt-level = 2` field still refuses qualification
before timing and leaves null conclusions. Unit checks also preserve zero,
other non-null values, unknown null hints, changed features and `mostly-unused`.

The initial public campaign stopped before timing on this representation
difference. Its null-result checkpoint is preserved. This comparison rule was
reviewed and tested before any public-workload timing or favorable result was
available; it does not retroactively convert the blocked run into evidence.
