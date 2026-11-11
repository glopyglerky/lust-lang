# Native composition diagnostics, 2026-11-11

The Lustr-owned compiler now composes local generic helpers with its unsafe
function contracts and native scalar library imports. The unchanged upstream
Lust tree remains pinned at `240e08d91d4aed0299639f6f11fd522912b248f9`.

## First failures and implementation

The optimized prior CI package at `ecbc7b6` rejected eight valid standalone
generic/unsafe compositions and the private-generic library producer. The
stock Rust oracle accepted them. [The first-failure report](mixed-generics-baseline.json)
retains those commands and diagnostics.

The implementation passes local parameters and bounds to the existing native
collector and monomorphizer. It additionally checks unsafe contracts in
templates that no caller instantiates. Template and trait-stub records do not
emit linked symbols; real native body/import symbol collisions still reject.
Generic exports and unsupported unsafe callable transport remain errors.

The owned grammar parser also borrows unchanged tokens rather than copying the
entire stream. It creates a replacement stream only when a supported extension
qualifier requires one. This removes an allocation and copy; no measured speed
gain is claimed here.

## Final correctness results

| Gate | Result | Raw evidence |
| --- | --- | --- |
| Mixed generic/unsafe and library composition | 22 source cases, 69 commands; ten valid and twelve invalid source cases; source-hidden consumers; real imported-symbol collision rejection | [mixed-generics-final.json](mixed-generics-final.json) |
| Existing and extended unsafe regressions | 21 source cases, 68 commands; invalid programs must reject with exit 1 | [unsafe-final.json](unsafe-final.json) |
| Required independent native libraries | 125 commands, 13 recorded checks; original required suite passed | [libraries-final.json](libraries-final.json) |
| Cargo/BlueFe compiler queries | 16 commands, 25 assertions | [protocol-final.json](protocol-final.json) |

All four final reports passed their functional checks, totaling 278 command
executions. The reports include compilation and runtime invocations by both
the candidate and oracle; that total is not a count of independent test cases.
The source-preservation guard also passed.

The strict acceptance evaluator passed 27 focused tests. A separate
[cold-runtime preflight](cold-runtime-preflight.json) ran six actual library
builds and six runtime checks with distinct initially empty temporary
directories. Each native sample produced its own TinyCC runtime object. This
checks cache isolation, not compilation speed. Earlier cold timing helpers
had left this independent cache shared through the system temporary directory.

## Exact scope

[source-identity.json](source-identity.json) binds the final reports, runner
sources, native compiler sources, protected upstream tree and executable hash.
The candidate was built with Rust 1.99.1 using the diagnostic gate profile,
optimization level 1, 16 codegen units and LTO disabled. It is not the optimized
package compiler and cannot qualify a delivered performance claim. The
baseline package was independently fetched from the successful `ecbc7b6` CI
artifact and its archive digest was verified before extraction.
The mixed-source inputs and historical runner are retained in `baseline-inputs`
and `final-inputs`. Compiled fixture executables are reproducible outputs and
are not embedded in this evidence directory.

The local executor cannot provide the required process trace or Unix-domain
daemon observations. These runs remain diagnostic and explicitly unqualified
for those requirements. CI must build the final optimized source and run the
traced library, relocation and strict native acceptance gates.

Full Rust compilation, complete upstream accepted-input coverage, general
artifact interoperability and both required speed floors remain unfinished.
No source fixture was reduced to make the full-Rust requirement pass.

## Subsequent optimized CI

The [optimized CI record](optimized-ci/README.md) preserves the results for
`c3d57f630cb4d2909e9217ae0fa074225376f76a`. Optimized correctness, relocation
and bounded Cargo concurrency checks passed. Strict speed and full Rust
acceptance failed, so that run produced no installable package archive.
