# Lustr native compiler development

Lustr starts directly from upstream Lust. The reference commit is recorded in
`upstream/lust.rev`; the Git ancestry preserves the original source history.
Native parsing, checking, lowering, generation and retained state remain one
compiler pipeline. Full Rust compatibility is the long-term goal.

## Extension rules

- Preserve upstream's valid accepted programs and Rust-compatible behavior.
- Implement missing behavior in the stage that already owns it. Keep the
  existing body, item and generic reuse paths active for unchanged inputs.
- Allocate extension state only where the new construct requires it. Track
  both successful and unsuccessful semantic lookups for invalidation.
- Compare edited retained sessions with fresh sessions and the Rust oracle.
  A source edit, configuration change or dependency change must never reuse
  an answer that is no longer valid.
- Measure the real optimized compiler against the pinned stock compiler with
  identical inputs, flags, runtime assets and cache conditions. Cold, no-op,
  body, signature, type, macro and invalid edits are separate results. Any
  regression or insufficient evidence leaves the speed requirement unproved.
- Preserve unsupported cases as explicit errors. Runtime fallback to another
  Rust compiler does not count as native language support.

## Decisions from upstream history

| Upstream change | Constraint on extensions |
| --- | --- |
| [Per-crate macro context](https://github.com/t3dotgg/lust-lang/commit/d5b533ba0f155de9645e209b5cfa3ac5b015ba3a) | New prelude and macro context belongs to each crate and the existing expansion cache. |
| [Narrow language-implementation reads](https://github.com/t3dotgg/lust-lang/commit/3087c60c7376afa091248d71733f4761b857c8f1) | New semantic dependencies should invalidate the readers of a changed fact, not every generic function. |
| [Dirty anonymous-constant selection](https://github.com/t3dotgg/lust-lang/commit/0a3b37800fd0e3cc3d1dc1bb2249392823cc3d7f) | Select affected work before constructing scopes or walking unchanged nodes. |
| [Stable module and negative lookup identities](https://github.com/t3dotgg/lust-lang/commit/ba15b7271a99928e5b4abcf6427b9c6ba3a24c5d) | Cache keys must describe semantic identity and absence, including configuration changes. |
| [Protection against root-name capture](https://github.com/t3dotgg/lust-lang/commit/90180efcb8d112956581bc34b2777902e728dfe2) | Qualified paths must retain their resolved definition when a nearer item has the same name; removing the rejection alone would restore wrong binding. |
| [Removal of an ineffective precheck](https://github.com/t3dotgg/lust-lang/commit/89974f0d41d045a1c0af73909ea2788cad39ebac) | A second classification pass needs measured benefit; fewer checked operations alone do not prove speed. |
| [Cold benchmark cache isolation](https://github.com/t3dotgg/lust-lang/commit/2511b82454f63d7a5fb0dca6a42963067ae8e9a1) | Disable daemon and disk reuse for cold measurements, and measure retained sessions separately. |

Upstream PRs are reviewed as proposed changes, not assumed-correct releases.
[PR #11](https://github.com/t3dotgg/lust-lang/pull/11) repairs library selection by
Cargo target kind and is retained with its original authorship. The clone-bound
fix in [PR #12](https://github.com/t3dotgg/lust-lang/pull/12) requires its own
reproduction and qualification. [PR #13](https://github.com/t3dotgg/lust-lang/pull/13)
changes diagnostic suggestions rather than native language coverage.

## Rust library boundaries

Rust's `no_std` attribute changes the implicit library and prelude to `core`.
Explicit `extern crate std` remains valid; explicit `alloc` requires its own
runtime support. A correct core prelude must not silently expose allocation or
OS APIs through builtin shortcuts. See the
[Rust Reference](https://doc.rust-lang.org/reference/names/preludes.html).

Language-level `no_std`, an allocation-free object, a freestanding executable,
and a supported target are distinct claims. Each needs its own output and
execution evidence. A hosted test harness can exercise a `no_std` library
without establishing freestanding startup, panic handling or target support.

Upstream resolves supported dependency source into a native workspace session.
That architecture is retained. A new crate-artifact format or compiler-routing
layer is not a prerequisite for extending the supported language.

## Current qualification

The implementation is under development. Tests and performance results are
scoped to the exact recorded revisions, inputs and hosts. Neither full Rust
coverage nor a universal no-regression claim is established by a finite suite.
The existing upstream release command can invoke Cargo/rustc when no native
foreign-target release path is selected. Such runs remain reference or hosted
compatibility results and cannot qualify native Lustr coverage.

## Native build artifacts

CI retains the exact native compiler binaries and their install-relative
library and TinyCC inputs for each native Linux and macOS host, on x86_64 and
AArch64. Artifacts named `native-build-<target>-<revision>` contain a tar archive
and its SHA256 checksum. After extraction, invoke `<target>/bin/lust`; keep the
sibling `lib/lust` directory with it. `build/identity.json` records the source
tree, bootstrap compiler, release profile, TinyCC revision and binary hashes.
`SHA256SUMS` records every packaged input. Packaging does not rebuild or strip
the compiler.

An artifact records a build, not a passing qualification. Read the same
revision's native gate and matched performance results before relying on it.
Foreign-target layout checks exercise the front end only; they do not establish
cross-linking or execution on that target. Native host gates execute the
generated programs and retain their comparison logs.
