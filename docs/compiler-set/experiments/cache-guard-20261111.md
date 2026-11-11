# Cache guard experiment result, 2026-11-11

## Decision

Candidate v4 remains **unadmitted and unpackaged**. It corrected the tested stale external-input results, but did not satisfy the preregistered requirement to be faster. Its aggregate candidate/baseline elapsed-time ratio was **0.9968085300458029**, with a stratified bootstrap 95% interval of **[0.982050529993242, 1.0108139619989267]**. The rule required the interval's upper bound to be below 1.

The workflow completed successfully because it finished the experiment and retained the evidence. A successful workflow does not admit the patch. No rerun or retuning contributed to this decision. A separate lexical-error cache-bypass risk remains unmeasured.

## Evidence

- [GitHub Actions run 42987550569](https://github.com/glopyglerky/lustr/actions/runs/42987550569), commit `298a83e61de927c459e6256ec6e85664e6b779ae` on `experiment/cache-guard-20261111`.
- [Evidence artifact 13842165101](https://github.com/glopyglerky/lustr/actions/runs/42987550569/artifacts/13842165101), 677,754 bytes and 403 files. Its downloaded ZIP SHA-256 matches GitHub's recorded digest, `08337147b8409869ac28d1e7d704839fc0b3831bbe9ed3fac3323f48c676a8cc`.
- [Pinned experiment plan, patch and scripts](https://github.com/glopyglerky/lustr/tree/298a83e61de927c459e6256ec6e85664e6b779ae/tools/experiments/cache-guard). The artifact retains their hashes, actual patch, compiler hashes, command logs, raw samples and reports.
- [Machine-readable conclusion](cache-guard-20261111.json) preserves all 1,120 raw timing rows in their original order, the preregistration, full-precision aggregate and per-case statistics, source identities, remaining gaps and evidence-file hashes. No sample was removed and no statistic was recomputed.

## Source and compiler bindings

Both compilers came from Lustr parent `ec89ca9342f50c77d2e1fd2ced2fc0c87829faab`. The protected native tree was `0ab1b09815802894e6078de0af59273cbdf46cb0`, matching Lust `240e08d91d4aed0299639f6f11fd522912b248f9`. Source guards passed on the primary checkout and baseline before and after the experiment. Only an external candidate worktree had a tracked change, in `compiler/native/src/driver/cache.rs`.

| Object | SHA-256 |
| --- | --- |
| Baseline cache source | `aa776daaebd1f5d1553c0b2a3927599692a41a184799f40ca91dddf47982eeae` |
| Candidate cache source | `e2103ffaddc09df08a854caeca712e641f48a5ae78bf998668d7d7d38fa6196f` |
| Candidate patch | `e44cc1d28c233931f6203bf46ccdd1946b0beba9b5927068da6dcd19422726b1` |
| baseline `lust` | `627373c5ca94390a57bb30fc84907241b7b9e816c79ea44c868b73447a4fbd46` |
| baseline `lust-core` | `38de17c180e6537e569d387213de61659f183775998fac98d83f56fc53f23b29` |
| candidate `lust` | `f4f7a3abacff7e7c7d482b0be4fae70e11439808e2aa8c8df86c7f22e49c3716` |
| candidate `lust-core` | `ac643c386e44a47fd3601fb5d1c030a6ec7bd45ca74d54ce37ce9451a27464b6` |

Both benchmark compilers were built sequentially with Rust 1.99.1, release optimization level 3, one codegen unit and ThinLTO. Both baked the same TinyCC installation, revision `43c7708b85681a2fd4451c8a541af4494a8919b2`. The evidence records the installed runtime file hashes and build-script paths. Cache unit tests used the separate gate profile; the timings below used the release binaries.

## Correctness and coverage

The candidate matched the Rust oracle on all **102 observations across 33 cases**, using **504 recorded commands**. The pristine baseline disagreed on **44 observations** after external inputs changed. These controls cover `env!`, `option_env!`, `include_str!` and `include_bytes!` with adjacent calls, spaces, line comments, nested comments and long trivia. They also cover generated macro calls, earlier quote/comment counterexamples and panic destructor order. `include!` is checked by the focused unit test, not by an end-to-end changing-include fixture in this run.

The tested ordinary variable and string lookalikes retained their whole-request cache behavior. All seven focused cache unit tests passed. All **18 daemon checks** passed, nine per compiler, across three macros and three external states. The daemon checks used comment-separated calls; they did not reproduce the baseline process-cache defect.

The independent native qualification comparison checked **12 fixtures** with matching manifest, runner and source hashes. Every baseline pass remained a candidate pass. The five passing cases were existing generics/traits, invalid type, invalid move, invalid alias and `offset_of!`. Seven expansion cases still failed native coverage in both lanes: unsafe pointer, async/await, generic associated type, dynamic iterator, return-position `impl Trait` in a trait, external function and native rlib. This experiment adds no full-Rust or Cargo compatibility claim.

## Paired performance results

Each scenario had **40 alternating baseline/candidate pairs**, giving **1,120 measured compiler commands**. Scenario order rotated between rounds. Both lanes used one CPU, one Lust thread, no daemon, the same source and flags, and the specified fresh or seeded cache state. Every successful output ran and matched the Rust oracle. Expected invalid edits and repeated invalid requests produced the required rejection. Wrong stale baseline answers were excluded from the performance comparison.

Ratios below are candidate time divided by baseline time. A ratio below 1 favors the candidate. Intervals describe the paired geometric-mean ratio. These observations are scoped to this host and these fixtures; they assume temporal stability within each alternating pair.

| Scenario | Baseline median, ms | Candidate median, ms | Geometric-mean ratio | Bootstrap 95% interval |
| --- | ---: | ---: | ---: | ---: |
| `fresh-existing` | 70.359588 | 70.214257 | 0.960479 | [0.916924, 0.997553] |
| `request-existing` | 6.301943 | 5.973912 | 0.941124 | [0.864929, 0.998229] |
| `body-edit` | 38.192792 | 38.812480 | 0.978961 | [0.915802, 1.026354] |
| `signature-edit` | 32.975262 | 33.283000 | 1.020040 | [0.990437, 1.051360] |
| `invalid-edit` | 8.997591 | 8.961956 | 0.997079 | [0.967959, 1.026407] |
| `invalid-request` | 5.215980 | 5.246068 | 0.994836 | [0.970804, 1.018120] |
| `early-option-env` | 31.245956 | 31.497328 | 1.048673 | [0.985737, 1.145836] |
| `variable-space` | 5.961808 | 6.090843 | 0.982830 | [0.879961, 1.053113] |
| `variable-comment` | 5.856519 | 5.968011 | 1.028894 | [0.989093, 1.075692] |
| `variable-long-comment` | 5.823457 | 5.758031 | 1.012295 | [0.971833, 1.060530] |
| `literal-string` | 5.859437 | 6.078481 | 1.016469 | [0.975986, 1.060132] |
| `fresh-logscan` | 392.606086 | 393.972639 | 1.008286 | [0.990871, 1.029154] |
| `request-logscan` | 8.761895 | 8.570921 | 0.947554 | [0.860120, 1.012261] |
| `generated-option-env` | 32.364469 | 32.503162 | 1.024443 | [0.988653, 1.064530] |

The aggregate gave equal weight to each scenario. No scenario met the preregistered definition of a confirmed regression greater than 5%, which required its interval's lower bound to exceed 1.05. That does not establish that no regression exists. Several point estimates are slower and their intervals remain broad. The aggregate faster rule failed.

The repeated-invalid-request case reused seeded caches, but rejection responses do not print the success-path whole-request-hit marker. Its zero marker count is not a demonstrated absence of caching. It is a repeated-rejection timing result.

## Separate unmeasured lexical-error risk

```rust
fn main(){ let _ = "env !"; let _ = "unterminated; }
```

A separate local pristine-baseline control returned `L0001` on two `lust check` invocations and created and retained a whole-request cache entry. This source's `env !` text is inside a string. The candidate's possible-call scan invokes the original lexer, which fails on the later unterminated string. Its conservative error branch then disables the request cache.

That can increase repeated rejection latency. The candidate's actual latency for this input was not measured, and this input is outside the preregistered fourteen-case performance set. The machine-readable conclusion includes the baseline source and diagnostic evidence. Do not interpret the ordinary variable/string controls or the aggregate result as resolving this case.

Simply treating every lexer error as cache-safe would require a correctness argument about diagnostic ordering and chunked lexing, including real external macros before or after a lexical error. This experiment makes no such change. It leaves the primary protected upstream source unchanged and does not publish a native package.
