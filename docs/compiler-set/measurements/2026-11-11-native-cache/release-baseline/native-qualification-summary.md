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
| Requested source | 8110d9f7aad82a8c382ebadcc092248f0db917bc |
| Measured source | 8110d9f7aad82a8c382ebadcc092248f0db917bc |
| Source tree | c523d6cc24f32776b8d89e2719738d652124eda6 |
| Evidence schema | lustr.native-speed-qualification.v1 |
| Native receipt schema | lustr.native-cargo-build.v2 |
| Native profile | release |
| Native build settings | {"codegen_units": 1, "debug": "line-tables-only", "incremental": false, "lto": "thin", "opt_level": 3} |
| Runner | {"cpu_affinity": \[0, 1, 2, 3\], "platform": "Linux-7.0.0-1017-azure-x86_64-with-glibc2.39"} |

| Executable | Version | SHA-256 |
| --- | --- | --- |
| native | lustr-native 0.1.0 (Lust 0.2.0, build 0fa2f406587a1041) | 37e7830c29e85a6e72549e3f3f23dbde567c463484c335b69e7f1c9eb00bbd67 |
| lust | lust 0.2.0 (build 861bd8ad) | df41ae4636c4eac79a44efeb595866f259124b0652d1bdfd8c2e9a5e982f7316 |
| cargo | cargo 1.99.1 (5f94df478 2026-08-27) | a670e02cfa39ac2e53f77a2d911ca1fc9e2c479b0603ec956625be1daf24e3af |
| rustc | rustc 1.99.1 (58cf8f9c3 2026-10-13) | b53b780f77f952062f8ca8a3b70aef2390b1e45ad91939be94afdc9543026853 |
| rustdoc | rustdoc 1.99.1 (58cf8f9c3 2026-10-13) | 832e53ca262db3d3fcaca659dc6e71f0f5a924a48adc580ee4278acac75f3e80 |
| lust-core | unavailable | 4ca2678317abe22d6bf2a88329db65645d8ffc4d0c14e1994b3f25b4ab1ca017 |

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
| lust/process-cold | failed | 12 | 43.706 | 43.672 | 0.955 | 1.015 | 1.084 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/process-cold/compile-plus-run | failed | 12 | 45.346 | 44.902 | 0.954 | 1.011 | 1.080 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/warm-noop | failed | 12 | 4.794 | 24.265 | 4.868 | 5.255 | 5.812 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/warm-noop/compile-plus-run | failed | 12 | 6.189 | 25.614 | 4.030 | 4.305 | 4.823 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/body-edit | failed | 12 | 6.159 | 26.989 | 3.217 | 4.282 | 4.791 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/body-edit/compile-plus-run | failed | 12 | 7.509 | 28.239 | 2.838 | 3.683 | 4.081 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/signature-edit | failed | 12 | 12.148 | 24.861 | 1.959 | 2.053 | 2.308 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/signature-edit/compile-plus-run | failed | 12 | 13.515 | 26.115 | 1.855 | 1.939 | 2.171 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/type-edit | failed | 12 | 11.519 | 24.037 | 1.969 | 2.099 | 2.270 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/type-edit/compile-plus-run | failed | 12 | 12.905 | 25.275 | 1.861 | 1.973 | 2.117 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/macro-edit | failed | 12 | 6.818 | 26.220 | 3.866 | 3.932 | 4.152 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/macro-edit/compile-plus-run | failed | 12 | 8.173 | 27.469 | 3.390 | 3.449 | 3.674 | at least one observed candidate invocation is slower; no outliers were discarded |
| lust/cold-invalid | failed | 12 | 6.389 | 6.042 | 0.930 | 0.977 | 1.172 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/fresh-target | passed | 12 | 81.715 | 66.087 | 0.783 | 0.827 | 0.890 | unavailable |
| cargo/fresh-target/compile-plus-run | passed | 12 | 82.706 | 67.179 | 0.786 | 0.829 | 0.891 | unavailable |
| cargo/no-change | passed | 12 | 37.976 | 18.410 | 0.506 | 0.546 | 0.791 | unavailable |
| cargo/no-change/compile-plus-run | passed | 12 | 38.883 | 19.526 | 0.522 | 0.561 | 0.803 | unavailable |
| cargo/body-edit | passed | 12 | 82.250 | 45.786 | 0.550 | 0.562 | 0.596 | unavailable |
| cargo/body-edit/compile-plus-run | passed | 12 | 83.294 | 47.298 | 0.559 | 0.572 | 0.603 | unavailable |
| cargo/signature-edit | passed | 12 | 80.595 | 45.428 | 0.560 | 0.584 | 0.666 | unavailable |
| cargo/signature-edit/compile-plus-run | passed | 12 | 81.637 | 46.867 | 0.570 | 0.595 | 0.673 | unavailable |
| cargo/type-edit | passed | 12 | 80.478 | 45.800 | 0.565 | 0.572 | 0.597 | unavailable |
| cargo/type-edit/compile-plus-run | passed | 12 | 81.456 | 46.910 | 0.572 | 0.579 | 0.603 | unavailable |
| cargo/macro-edit | failed | 12 | 82.240 | 49.135 | 0.731 | 0.967 | 3.568 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/macro-edit/compile-plus-run | failed | 12 | 83.261 | 50.292 | 0.737 | 0.972 | 3.539 | at least one observed candidate invocation is slower; no outliers were discarded |
| cargo/cold-invalid | passed | 12 | 56.174 | 25.570 | 0.450 | 0.458 | 0.479 | unavailable |
| mixed/fresh-target | failed | 12 | 83.152 | 83.399 | 0.990 | 1.010 | 1.056 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/fresh-target/compile-plus-run | failed | 12 | 84.142 | 84.542 | 0.992 | 1.012 | 1.059 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/no-change | passed | 12 | 28.359 | 16.636 | 0.580 | 0.591 | 0.630 | unavailable |
| mixed/no-change/compile-plus-run | passed | 12 | 29.305 | 17.739 | 0.601 | 0.611 | 0.653 | unavailable |
| mixed/body-edit | passed | 12 | 65.324 | 49.911 | 0.748 | 0.763 | 0.799 | unavailable |
| mixed/body-edit/compile-plus-run | passed | 12 | 66.363 | 51.075 | 0.754 | 0.769 | 0.804 | unavailable |
| mixed/signature-edit | passed | 12 | 64.574 | 48.948 | 0.746 | 0.770 | 0.843 | unavailable |
| mixed/signature-edit/compile-plus-run | passed | 12 | 65.575 | 50.109 | 0.752 | 0.776 | 0.845 | unavailable |
| mixed/type-edit | failed | 12 | 67.489 | 49.254 | 0.768 | 0.853 | 1.390 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/type-edit/compile-plus-run | failed | 12 | 68.592 | 50.416 | 0.774 | 0.857 | 1.386 | at least one observed candidate invocation is slower; no outliers were discarded |
| mixed/macro-edit | passed | 12 | 70.137 | 49.748 | 0.696 | 0.762 | 0.985 | unavailable |
| mixed/macro-edit/compile-plus-run | failed | 12 | 71.266 | 50.990 | 0.703 | 0.771 | 1.010 | at least one observed candidate invocation is slower; no outliers were discarded |
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
- cargo/macro-edit
- cargo/macro-edit/compile-plus-run
- mixed/fresh-target
- mixed/fresh-target/compile-plus-run
- mixed/type-edit
- mixed/type-edit/compile-plus-run
- mixed/macro-edit/compile-plus-run
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
