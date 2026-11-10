# Preserve upstream Lust

Lust must remain unchanged from upstream. Lustr supplies the missing Rust
support in separately owned code. Any proposed Lust divergence must demonstrate
both why it is necessary and that it produces faster results. The
[native compiler contract](native-contract.md) remains the full-project goal.

## Exact source baseline

[`upstream/lust.pin`](../../upstream/lust.pin) pins
`t3dotgg/lust-lang` revision `240e08d91d4aed0299639f6f11fd522912b248f9`,
tree `0ab1b09815802894e6078de0af59273cbdf46cb0`, at `compiler/native`.
The upstream default branch was verified to point to that revision.

The protected tree includes the original compiler, libraries, build files,
tests, documentation, research probes and metadata. Moving it under a prefix
does not alter its tracked contents. Required root-level submodule routing is
Lustr integration metadata; gitlink targets remain pinned references. External
submodule contents and generated dependencies need their own qualification.

Earlier fork overrides and an unpublished coverage-expansion candidate are
preserved in history or restricted research records. They are not admitted to
the active Lust baseline. Passing correctness tests or demonstrating additional
coverage alone does not meet the faster-results requirement.

## Source guard

The dependency-free Rust guard compares the complete committed or staged Lust
tree with the pinned upstream tree, then rejects tracked worktree edits and
untracked additions in that scope. Changes to independently owned project files
do not fail the guard. CI runs the committed-tree check and its drift tests.

Build the helper outside the source tree and run it from the repository root:

```sh
rustc --edition=2024 tools/lust-upstream-check.rs -o /absolute/cache/lust-upstream-check
/absolute/cache/lust-upstream-check
```

`--index` checks a staged restoration before its commit. The check uses Git tree
identity and scoped Git status. It does not certify ignored build artifacts,
external dependencies, compiler correctness, or universal source/secret absence.

## Admission evidence for a proposed divergence

No exception is currently enabled. Before changing that decision, retain:

1. The exact upstream revision, minimal proposed patch and affected interface.
2. A reproducible need, including the failing unmodified-upstream case and why
   the proposed change belongs in Lust rather than Lustr-owned implementation.
3. Acceptance, rejection, runtime, ownership, panic/drop and warm/cold edit
   evidence for the changed native behavior.
4. Preregistered equivalent-workload comparisons of pristine upstream and the
   candidate, including exact builds, semantic settings, cache state, raw
   observations, regressions and replayable analysis. A failed compilation is
   not a faster successful build. Comparing only against rustc cannot establish
   a speed improvement over Lust.
5. A justified faster-results conclusion with explicit mathematical or
   statistical assumptions and a stated scope. Full Rust coverage and the
   preservation of Lust's existing fast path remain separate requirements.

An accepted future exception must have an explicit, reviewable record and guard
support. A label, source relocation, correctness-only result or promised future
benchmark cannot admit it.
