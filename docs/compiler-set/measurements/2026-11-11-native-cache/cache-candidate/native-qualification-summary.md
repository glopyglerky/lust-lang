# Native compiler qualification

**UNQUALIFIED. Failed, missing or inconclusive evidence blocks the installable archive.**

Cold and warm speed floors both apply. No faster mode offsets a slower mode.
Hosted-runner timings include environmental noise. This report proves no universal speed or language-coverage claim.

## Summary consistency checks

- Workflow check native_speed did not report success.

These checks annotate the evidence. They do not inspect the executable or replace the release admission gate.

## Source and compiler builds

| Field | Value |
| --- | --- |
| Requested source | 5b98ffb63da0139d778ef4a26de28b1d38fd4a0e |
| Measured source | 5b98ffb63da0139d778ef4a26de28b1d38fd4a0e |
| Source tree | 54a630dc7d850871cd7921340c15bcea63cb3bf4 |
| Evidence schema | lustr.native-speed-qualification.v1 |
| Native receipt schema | lustr.native-cargo-build.v2 |
| Native profile | release |
| Native build settings | {"codegen_units": 1, "debug": "line-tables-only", "incremental": false, "lto": "thin", "opt_level": 3} |
| Runner | {"cpu_affinity": \[0, 1, 2, 3\], "platform": "Linux-7.0.0-1017-azure-x86_64-with-glibc2.39"} |

| Executable | Version | SHA-256 |
| --- | --- | --- |
| native | lustr-native 0.1.0 (Lust 0.2.0, build b471a757620e224f) | 293f940c6343a51d8b208809815549e5b4f6233c341966d736bce5f740b291f7 |
| lust | lust 0.2.0 (build 861bd8ad) | df41ae4636c4eac79a44efeb595866f259124b0652d1bdfd8c2e9a5e982f7316 |
| cargo | cargo 1.99.1 (5f94df478 2026-08-27) | a670e02cfa39ac2e53f77a2d911ca1fc9e2c479b0603ec956625be1daf24e3af |
| rustc | rustc 1.99.1 (58cf8f9c3 2026-10-13) | b53b780f77f952062f8ca8a3b70aef2390b1e45ad91939be94afdc9543026853 |
| rustdoc | rustdoc 1.99.1 (58cf8f9c3 2026-10-13) | 832e53ca262db3d3fcaca659dc6e71f0f5a924a48adc580ee4278acac75f3e80 |
| lust-core | unavailable | a12659c6e3a2a74e93ee843a33fe329e0475ad41358bb483ad3033fc8c6a82fb |

## Workflow steps

| Step | Outcome |
| --- | --- |
| dependencies | success |
| source_before | success |
| invocations | success |
| acceptance_tests | success |
| native_build | success |
| artifacts | success |
| protocol | success |
| unsafe | success |
| mixed_generics | success |
| vendor | success |
| request_cache | success |
| safe_library | success |
| package | success |
| relocation | success |
| upstream_coverage | success |
| native_speed | failure |
| source_after | success |

## Compatibility and evidence requirements

| Requirement | State | Detail |
| --- | --- | --- |
| identities | passed | unavailable |
| provenance | passed | unavailable |
| input-stability | passed | unavailable |
| upstream-preservation | passed | Pinned source identity only; accepted-input preservation has a separate requirement. |
| upstream-accepted-corpus | unproved | Representative direct fixtures do not cover the complete upstream accepted corpus. No complete preservation claim is authorized. |
| engine-trace | passed | unavailable |
| concurrent-cargo | passed | unavailable |
| mixed-compatibility | passed | unavailable |
| full-rust-compatibility | failed | unavailable |
| build-scripts | unproved | unavailable |
| doctests | unproved | unavailable |
| full-rust/fresh-target | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 317}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 318}} |
| full-rust/no-change | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 320}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 321}} |
| full-rust/check | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 323}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 324}} |
| full-rust/release | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 325}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 326}} |
| full-rust/compile-plus-test | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 328}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 329}} |
| full-rust/doctests | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 330}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 331}} |
| full-rust/type-edit | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 332}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 333}} |
| full-rust/macro-edit | failed | unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 335}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 336}} |

## Every timing mode

Durations are medians in milliseconds. Ratios are candidate / baseline; the limit is 1.000.
Upper bounds and paired ratios are taken from the report without dropping outliers or recomputing decisions.

| Mode | State | Pairs | Baseline ms | Lustr ms | Paired ratio | Upper 95% | Worst pair | Detail |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| lust/process-cold | failed | 12 | 50.116 | 50.840 | 1.006 | 1.032 | 1.127 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/process-cold/compile-plus-run | failed | 12 | 51.249 | 51.965 | 1.006 | 1.031 | 1.124 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/fresh-cache | failed | 12 | 51.396 | 52.408 | 1.010 | 1.033 | 1.089 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/fresh-cache/compile-plus-run | failed | 12 | 52.544 | 53.548 | 1.010 | 1.032 | 1.088 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/warm-noop | failed | 12 | 4.681 | 4.407 | 0.960 | 1.680 | 24.909 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/warm-noop/compile-plus-run | failed | 12 | 5.922 | 5.685 | 0.964 | 1.617 | 19.991 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/body-edit | failed | 12 | 6.339 | 28.869 | 4.542 | 4.697 | 4.967 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/body-edit/compile-plus-run | failed | 12 | 7.636 | 30.026 | 3.939 | 4.061 | 4.287 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/signature-edit | failed | 12 | 12.864 | 28.217 | 1.861 | 2.173 | 2.291 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/signature-edit/compile-plus-run | failed | 12 | 14.202 | 29.427 | 1.774 | 2.055 | 2.160 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/type-edit | failed | 12 | 12.897 | 28.145 | 2.280 | 2.514 | 3.994 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/type-edit/compile-plus-run | failed | 12 | 14.216 | 29.308 | 2.153 | 2.369 | 3.718 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/macro-edit | failed | 12 | 6.959 | 28.289 | 4.034 | 4.154 | 4.434 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/macro-edit/compile-plus-run | failed | 12 | 8.266 | 29.497 | 3.551 | 3.644 | 3.883 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/cold-invalid | failed | 12 | 6.276 | 5.602 | 0.933 | 0.989 | 1.107 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/fresh-target | failed | 12 | 81.878 | 75.252 | 0.917 | 0.951 | 1.090 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/fresh-target/compile-plus-run | failed | 12 | 82.835 | 76.320 | 0.919 | 0.954 | 1.102 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/no-change | passed | 12 | 37.129 | 19.142 | 0.511 | 0.527 | 0.560 | unavailable |
| cargo/no-change/compile-plus-run | passed | 12 | 38.014 | 20.211 | 0.527 | 0.542 | 0.575 | unavailable |
| cargo/body-edit | passed | 12 | 80.225 | 50.709 | 0.628 | 0.638 | 0.666 | unavailable |
| cargo/body-edit/compile-plus-run | passed | 12 | 81.136 | 51.816 | 0.635 | 0.645 | 0.673 | unavailable |
| cargo/signature-edit | failed | 12 | 80.960 | 49.617 | 0.573 | 0.661 | 1.171 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/signature-edit/compile-plus-run | failed | 12 | 81.895 | 50.743 | 0.579 | 0.667 | 1.171 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/type-edit | failed | 12 | 81.661 | 50.312 | 0.733 | 1.053 | 5.478 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/type-edit/compile-plus-run | failed | 12 | 82.625 | 51.453 | 0.740 | 1.060 | 5.429 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/macro-edit | passed | 12 | 81.249 | 49.823 | 0.615 | 0.630 | 0.701 | unavailable |
| cargo/macro-edit/compile-plus-run | passed | 12 | 82.175 | 50.949 | 0.622 | 0.637 | 0.707 | unavailable |
| cargo/cold-invalid | passed | 12 | 54.516 | 26.423 | 0.480 | 0.490 | 0.520 | unavailable |
| mixed/fresh-target | failed | 12 | 83.050 | 88.945 | 1.078 | 1.099 | 1.196 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/fresh-target/compile-plus-run | failed | 12 | 83.971 | 90.087 | 1.079 | 1.101 | 1.196 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/no-change | failed | 12 | 28.114 | 16.993 | 0.672 | 0.960 | 4.876 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/no-change/compile-plus-run | failed | 12 | 29.025 | 18.002 | 0.690 | 0.977 | 4.762 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/body-edit | failed | 12 | 67.034 | 53.621 | 0.848 | 0.988 | 1.998 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/body-edit/compile-plus-run | failed | 12 | 67.981 | 54.774 | 0.852 | 0.992 | 1.987 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/signature-edit | passed | 12 | 67.360 | 52.425 | 0.779 | 0.797 | 0.845 | unavailable |
| mixed/signature-edit/compile-plus-run | passed | 12 | 68.313 | 53.558 | 0.784 | 0.802 | 0.849 | unavailable |
| mixed/type-edit | failed | 12 | 68.380 | 52.494 | 0.814 | 0.986 | 2.392 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/type-edit/compile-plus-run | failed | 12 | 69.385 | 53.613 | 0.819 | 0.989 | 2.376 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/macro-edit | passed | 12 | 67.385 | 53.079 | 0.780 | 0.796 | 0.825 | unavailable |
| mixed/macro-edit/compile-plus-run | passed | 12 | 68.494 | 54.217 | 0.785 | 0.800 | 0.829 | unavailable |
| full-rust/fresh-target | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/fresh-target/compile-plus-run | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/no-change | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/no-change/compile-plus-run | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/check | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/release | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/release/compile-plus-run | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/compile-plus-test | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/doctests | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/type-edit | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/type-edit/compile-plus-run | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/macro-edit | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |
| full-rust/macro-edit/compile-plus-run | unproved | 0 | unavailable | unavailable | unavailable | unavailable | unavailable | required measurements missing or functional compilation failed |

## Blocking requirements

- upstream-accepted-corpus
- full-rust-compatibility
- build-scripts
- doctests
- lust/process-cold
- lust/process-cold/compile-plus-run
- lust/fresh-cache
- lust/fresh-cache/compile-plus-run
- lust/warm-noop
- lust/warm-noop/compile-plus-run
- lust/body-edit
- lust/body-edit/compile-plus-run
- lust/signature-edit
- lust/signature-edit/compile-plus-run
- lust/type-edit
- lust/type-edit/compile-plus-run
- lust/macro-edit
- lust/macro-edit/compile-plus-run
- lust/cold-invalid
- cargo/fresh-target
- cargo/fresh-target/compile-plus-run
- cargo/signature-edit
- cargo/signature-edit/compile-plus-run
- cargo/type-edit
- cargo/type-edit/compile-plus-run
- mixed/fresh-target
- mixed/fresh-target/compile-plus-run
- mixed/no-change
- mixed/no-change/compile-plus-run
- mixed/body-edit
- mixed/body-edit/compile-plus-run
- mixed/type-edit
- mixed/type-edit/compile-plus-run
- full-rust/fresh-target
- full-rust/fresh-target/compile-plus-run
- full-rust/no-change
- full-rust/no-change/compile-plus-run
- full-rust/check
- full-rust/release
- full-rust/release/compile-plus-run
- full-rust/compile-plus-test
- full-rust/doctests
- full-rust/type-edit
- full-rust/type-edit/compile-plus-run
- full-rust/macro-edit
- full-rust/macro-edit/compile-plus-run
- retained execution failures

## Retained execution failures

- full-rust/fresh-target: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 317}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 318}}
- full-rust/no-change: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 320}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 321}}
- full-rust/check: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 323}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 324}}
- full-rust/release: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 325}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 326}}
- full-rust/compile-plus-test: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 328}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 329}}
- full-rust/doctests: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 330}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 331}}
- full-rust/type-edit: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 332}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 333}}
- full-rust/macro-edit: ValueError: unchanged workspace failed: {"candidate": {"correct": false, "reason": "Cargo command failed with exit 101", "sample": -1, "command": 335}, "baseline": {"correct": true, "reason": null, "sample": -1, "command": 336}}

Raw JSON, command output, traces and build receipts are retained in the workflow evidence artifact.
