# Reproducible paired compiler measurements

`crates/lustr-bench` is a standalone Rust workspace. It adds no dependencies to
`crates/lustr`. This runner qualifies explicitly supplied command vectors; it
neither selects a compiler automatically nor installs or changes defaults.
Wrapping unchanged Cargo does not itself accelerate compilation.

Build with the campaign's explicitly selected coordinator and produced compiler:

```text
<toolchain>/bin/lustr --bluefe-root <toolchain> build --locked \
  --manifest-path crates/lustr-bench/Cargo.toml --target-dir <build-output> -j 2
<build-output>/debug/lustr-bench <spec.json> <new-absolute-report-directory>
```

Use existing frozen snapshots, never canonical project trees. The output parent
must already exist; the report directory must not exist. Reports and lane target
directories must be disjoint from source. Each lane gets a newly created
`targets/<lane>` under the report directory. CLI target/config/jobs overrides
are rejected; use the allowlisted environment. This intentionally excludes
benchmark recipes needing explicit Cargo `--config`; checked source Cargo
configuration remains available. Each invocation clears the inherited environment.
List all required settings explicitly. Do not put credentials in argv, environment,
normalization rules, source snapshots, or logs. Commands are trusted experiment
inputs, not sandboxed programs: build scripts and tests can write files or access
external services. Supply only offline fixtures/workloads already authorized for
execution. Content guards detect changes; they do not undo arbitrary program effects.

A complete example is [`sample-spec.json`](../../crates/lustr-bench/sample-spec.json).
Replace placeholder paths and tool SHA256 values. JSON fields below are required
unless stated otherwise; unknown fields are rejected by the Rust deserializer.
`SPEC.json` is a UTF-8 JSON object. OS arguments and environment values accept a
string or `{"hex":"<even-length hexadecimal OS bytes>"}` on Linux; no shell,
interpolation, or lossy decoding is involved. File path fields use UTF-8 paths.

| Spec field | Type and meaning |
| --- | --- |
| `schema` | Integer `1`. |
| `source` | Absolute canonical frozen source directory outside canonical repositories. |
| `repository`, `revision` | Public upstream URL and pinned revision or complete fixture identity; declarations retained with content hashes. The runner does not fetch or authenticate revisions. |
| `package`, `root_crate` | Package label and actual rustc `--crate-name` root name. |
| `profile`, `target`, `features` | Shared labels; string, string, array of strings. Actual commands, verbose compiler evidence and Cargo fingerprints accompany these declarations. |
| `semantic_settings` | Nonempty string map declaring panic, debug, overflow, optimization and other relevant settings. |
| `external_inputs` | Array of absolute files/directories observed outside source, e.g. Cargo configuration, sysroot libraries, linker, build-script data. Directory hashes exclude a top-level `.git`; explicitly name Git input files if observed. |
| `scenario` | One timing label from the table below. |
| `force_root` | Optional/null safe relative root file whose modification time is changed before each command. Bytes are unchanged. |
| `snapshot_mtime_authorized` | Boolean; must be true for `force_root`. This never authorizes canonical source writes. |
| `pairs` | Even integer, 8–10000; even counts give balanced order. |
| `warmup_pairs` | Integer 1–100; excluded entirely from statistics. |
| `seed` | Unsigned 64-bit seed for ordering and bootstrap. |
| `bootstrap_resamples` | Integer 100–1000000. |
| `timeout_seconds` | Per command integer 1–86400. |
| `output_limit_bytes` | Combined stdout/stderr threshold, 1024–268435456 bytes. |
| `lanes` | Array of 2–8 lane objects; the first is baseline. |

| Lane field | Meaning |
| --- | --- |
| `name` | Unique nonempty ASCII alphanumeric/hyphen label. |
| `env` | Explicit allowlisted environment map. `CARGO`, `RUSTC`, `RUSTDOC` must name hash-pinned tools. Set a positive `CARGO_BUILD_JOBS` within the runner's available CPU affinity. `CARGO_TARGET_DIR` is runner-owned. |
| `tools` | Array of `{path, sha256, version_args}`; absolute executable path, expected SHA256, array of OS arguments for its version probe. Include coordinator, Cargo, rustc, rustdoc, wrappers and any direct gate executable. |
| `command` | Timed `{argv, expected_exit:0}`. Full argv includes absolute executable. |
| `selection` | Untimed command returning full `cargo metadata --format-version 1 --locked` JSON with a non-null resolve graph. Use identical feature selection. |
| `evidence` | Untimed verbose Cargo command forcing the root compiler to run; stdout/stderr retained. |
| `evidence_contains` | Nonempty identical array across lanes of nonempty actual root compiler flag strings, e.g. edition, debuginfo, panic/overflow flags where emitted. A single Cargo `Running` line must contain the root crate, exact RUSTC path, and every string. |
| `gates` | Array of `{name, kind, command, compare}`. Each lane needs `runtime` or `tests`, plus `rejection` expecting a nonzero code. `compare` is `stdout`, `stderr`, or `both`. Gate names/kinds/expected codes/streams must agree. Supply substantive project tests and expected-output assertions, including relevant panic/drop behavior; a successful compile is insufficient. |
| `normalize` | Ordered array of `[literal_string, replacement_string]` pairs applied to comparison bytes, never raw evidence. Typical use: replace each lane's target path with the same marker. Avoid erasing meaningful outputs. No regex or executable expressions. |

Environment allowlist: PATH, HOME, TMPDIR, LANG, LC_ALL, TZ, CARGO_HOME,
RUSTUP_HOME, CARGO, LUST_CARGO, RUSTC, RUSTDOC, RUSTFLAGS,
CARGO_ENCODED_RUSTFLAGS, RUSTDOCFLAGS, RUSTC_WRAPPER,
RUSTC_WORKSPACE_WRAPPER, CARGO_BUILD_JOBS, CARGO_BUILD_TARGET,
CARGO_INCREMENTAL, CARGO_NET_OFFLINE, CARGO_TERM_COLOR, LUST_NO_DAEMON,
LUST_NO_CACHE, LUST_CACHE_DIR, LUST_DAEMON_SOCKET, LUST_CRANELIFT_BACKEND,
CC, CXX, AR, CFLAGS, CXXFLAGS, PKG_CONFIG_PATH, and keys beginning
CARGO_PROFILE_ or CARGO_TARGET_. Reproducibility settings, wrappers, flags and
semantic environment must agree across lanes. Compiler/tool paths and scheduler
choices may differ. Cargo operation and all arguments after its subcommand must
match exactly between lanes for measurement, metadata and evidence. Thus package,
features, target and profile CLI differences are rejected rather than hidden.
An option like coordinator `--frontend-jobs N` belongs before the Cargo subcommand.

The operation parser recognizes `rustc` explicitly and skips selector option
values. `rustc --profile check` is a metadata-checking operation; the word
`check` in that profile value never hides preceding package or feature flags
from lane comparison. Primed checking requires observed metadata-only emission.
When a lane requests frontend jobs, its actual root compiler command must show
that same count. Planning commands and unsupported prefixes are rejected.

Before any build/evidence command, resolved Cargo metadata must report both
`target_directory` and `build_directory` inside the newly owned lane directory.
External or shared build directories, including symlink escapes, are rejected.
Deleting a lane therefore also removes its admitted intermediate build tree in
the clean-build scenario. Both legacy `.fingerprint` and newer `fingerprint`
Cargo layouts are collected, and an empty observed fingerprint set is refused.
This checks resolved build-directory configuration; it is not a sandbox against
arbitrary effects performed by a trusted build script or compiler wrapper.

The companion `lustr-bench-gate tests <Cargo arguments>` uses the explicitly
selected `CARGO`, requires an executing test command and a nonempty completed
test harness, and compares test names/counts while ignoring elapsed-time text.
Original stdout/stderr bytes are retained in the diagnostic stream with byte-count
boundaries. `--version` supplies a hash-bound identity probe. The `reject
<source>` gate uses the selected `RUSTC` with JSON diagnostics and requires exit 1
and a structured error diagnostic whose exact code is `E0502`. Filenames, source
snippets, rendered messages and explanatory text cannot supply that code. It
returns 1 only for that expected rejection. Validation errors return 42, so an
unexpected success or wrong failure cannot satisfy a rejection gate. Include
this helper's binary hash among the recorded tools. These gates are bounded
conformance checks, not proofs of arbitrary program equivalence.

For an explicit coordinator/backend route, use
`tests-via <absolute-lustr> --bluefe-root <absolute-root> --engine <auto|rustc|cranelift> test <Cargo arguments>`.
All three `CARGO`, `RUSTC` and `RUSTDOC` selectors must exactly match that root's
`bin` paths. The helper refuses a mismatch before execution and forwards the
remaining Cargo argument vector unchanged. It records the child status and raw
streams, then emits the same test comparison JSON as the direct-Cargo mode.
Include the coordinator's executable hash and the matching backend as a hashed
external input. The [route-binding statement and native checks](proofs/routed-test-gates.md)
describe the scope. A requested engine still needs actual compiler-command and
backend-load evidence; caller profiles/configuration retain their normal effect.
This permits scoped Cranelift test gates without granting the runner arbitrary
CLI `--config` overrides or adding Cargo profile backend environment overrides.

| Scenario | What it measures |
| --- | --- |
| `no-op` | Already primed target, no forced source timestamp change. |
| `primed-root-check` | Dependency-primed root checking after a byte-preserving root timestamp change. |
| `primed-root-build` | Dependency-primed root build after that change. |
| `compile-plus-test` | Explicit compile/test command; primed unless `force_root` requests a rebuild. |
| `clean-dependency-build` | Delete only the runner-owned lane target before each sample; source/dependency/OS caches remain warm. This is not a cold disk-cache measurement. |

Primed root scenarios require `force_root`. For any force mode, timed and warmup
commands must themselves use verbose Cargo output: the root compiler evidence
check applies to every sample. The root file's original atime/mtime is restored
on ordinary completion/failure. Abrupt termination cannot guarantee restoration.
A primed root check is never described as a whole-repository cold build.

Before timing, the runner checks tool binary hashes, records version outputs,
hashes source and external inputs, captures actual resolved dependency selection,
hashes selected dependency source trees outside the root, captures actual compiler
commands, and runs correctness gates. It compares normalized metadata and gate
results across lanes. The metadata comparison parses JSON and applies only the
versioned `cargo-optional-min-opt-level-none-v1` compatibility rule to Cargo
format version 1; other or missing versions are refused. A null
`packages[*].hints.min-opt-level` compares as an absent field. Each application
is recorded in `metadata_comparison_projections`; complete raw JSON and its hash
remain in the command records. Non-null values, other hints, unknown fields,
features and dependency graph values remain subject to equality. JSON object key
order has no meaning; array order remains significant. The
[conditional projection proof](proofs/metadata-compatibility.md) binds this rule
to the pinned Cargo source. No other null fields are removed.

All source files except top-level `.git` are included,
including manifests, Cargo.lock, hidden configuration and vendored dependencies;
paths are represented by their exact hex bytes and file values contain mode plus
content SHA256. Symlinks/special files are rejected: use a complete materialized
snapshot and record that preparation. Git metadata, sysroots, native libraries,
linker scripts, tool configuration and build-script observations outside those
source trees must be supplied as external inputs when relevant. The runner is
not a complete build-input tracer or a semantic equivalence proof.

After each sample, source/external-input and tool hashes are rechecked outside
timing. Final compiler evidence, correctness gates, selected dependency hashes
and source/tool guards must pass before any summary appears. Cargo `.fingerprint`
JSON is retained in the report with content hashes, including actual selected
feature/profile/dependency identifiers. Binary differences for Cargo and rustc
are automatically labeled as confounders with versions retained. Keep stock
Cargo/rustc versus produced BlueFe comparisons separate from same-compiler serial
versus parallel controls. Different compiler/Cargo versions remain confounders
regardless of passing tests.

New runs use `randomized-cyclic-blocks-v1`: each block visits all cyclic rotations
of a shuffled lane list, in shuffled rotation order. Every lane runs once per
round. Complete blocks of `L` rounds visit every execution position once per
lane. With a partial final block, counts for each lane's positions differ by at
most one. Choose an even number of rounds divisible by `L`, at least eight,
for exact position balance. The [construction and position theorem](proofs/order-balance.md)
state the deterministic guarantee and its limits. The report records the method
and every actual order. There is no selective dropping of slow samples.
One excluded warmup round is the minimum;
choose more before viewing results. Filesystem hashing, correctness work and
version probing lie outside timing, but can affect cache state. Record hardware,
OS, thermal/load context and cache preparation with the campaign evidence.

Wall time includes child creation and completion observation, using a monotonic
clock and 1 ms status polling. It excludes metadata touching, guards and final
descendant cleanup. This resolution makes tiny no-op results diagnostic only.
Commands have null stdin, separate raw output files, an isolated process group,
a timeout and a combined output threshold. `/bin/kill` is the explicit Linux host
compatibility utility used to terminate the group; no unsafe FFI is implemented.
Descendants that deliberately escape the group are outside this cleanup model.
Output is checked periodically, so a burst can exceed the threshold before the
child is killed. This is a cancellation threshold, not a hard filesystem quota.
RSS, energy and total process-group peak memory are not measured.

Each command records `host_before` and `host_after` outside its wall-time
interval. These contain the runner's allowed CPU list, the three load averages,
raw `/proc/stat` CPU counter rows and CPU/I/O pressure observations from
[Linux PSI](https://docs.kernel.org/accounting/psi.html). The final observation
follows descendant cleanup. These bracket observations are not continuous
monitoring or complete child-process accounting. Reading them may also affect
scheduling/cache state. Linux proc/PSI access is required for this runner.

Label each campaign's host model before timing. A shared-host exploratory study
retains existing services and background work, uses a fixed CPU set/budget and
records admission context plus every command observation. It does not satisfy a
quiet-host study's admission gate or establish isolated causal speedups. Retain
load spikes and unfavorable observations; do not remove samples after seeing
ratios. Bootstrap and randomization assumptions remain conditional under either
host model. Never stop unrelated services or jobs to manufacture admission.

The earlier forward/reverse method did not balance every position with three
or more lanes: the middle lane stayed in the middle in a three-lane campaign.
Published historical reports retain that limitation. The new schedule balances
positions, but does not balance every predecessor or cancel host/cache effects;
pairing and bootstrap resampling do not supply those guarantees.
Also verify the actual options used by correctness gates. A direct Cargo gate
does not automatically inherit frontend options supplied only to a coordinator's
timed `rustc` command. Passing that gate qualifies its actual invocation, not
every timed scheduling configuration. The
[2026-11-09 dataset](measurements/2026-11-09-shared-v2/README.md) records both limits.

## Report schema and publication

`report.json` schema 1 contains `runner_version`, `runner_sha256`,
`spec_sha256`, complete parsed `spec`, `status` (`passed` or `failed`), `order_method`, `records`,
`source_before`, `source_after` (when reached), `final_source_observation`,
`final_tools_observation`, `identities`, `dependency_sources_before/after`,
`compiler_or_cargo_identity_confounders`, `actual_cargo_fingerprints`, `order`,
`metadata_comparison_rule`, `metadata_comparison_projections`,
`memory_energy`, `inference_assumptions`, and `summaries`. Failure adds `error`,
sets `summaries` to null, and retains completed records and raw output. A rejected
spec/setup before output creation has no run report. OS-level failure to write a
report can leave `.pending` evidence. The exact input bytes are saved separately
as `spec.original.json`, whose hash equals `spec_sha256`.

Each record has `lane`, `phase`, optional/null `pair`, exact `argv`,
`wall_seconds`, `exit`, `signal`, `timed_out`, `output_limit`, raw relative
`stdout`/`stderr` filenames, their SHA256 digests and byte counts. Records include
warmups, identity probes, metadata, gates, compiler evidence and timed samples.
Failed commands are never retried under another compiler.

Each baseline/candidate summary contains `analysis`: method/version,
`pairs`, `raw_ratios`, `log_ratios`, `geometric_ratio`, `bootstrap_95`,
`baseline_median`, `candidate_median`, `baseline_p95`, `candidate_p95`,
`exact_sign_flip_p` (null above 16 pairs), `bootstrap_resamples`, and `seed`.
Time summaries use seconds; ratios above one favor the candidate. Report values
are observations, not an automatically authorized headline speedup. Review gates,
inputs, flags, version confounders and assumptions first. Statistical details and
proof scope are in [benchmark-analysis.md](proofs/benchmark-analysis.md).

Reports use a new directory and atomic no-overwrite hard-link publication after
fsync; they never overwrite an earlier report. Raw reports are private and can
contain local paths, command output and source names. Do not commit them as-is.
When producing public sanitized evidence, retain the original privately, hash the
sanitized payload bytes separately, update every rewritten file reference and
digest, and identify original-payload hashes distinctly. Never represent an
original hash as a hash of rewritten bytes. Public docs and the example use
portable placeholders rather than private host paths.

## Small-fixture validation and campaign handoff

The opt-in integration test copies the public in-tree `fixtures/tiny` to its own
output snapshot, runs eight paired root checks with the same produced BlueFe in
both lanes, executes tests, validates runtime/unwind with one destructor call and
expected E0502 rejection, and checks no-overwrite and retained failure behavior.
It also refuses unequal features hidden before `--profile check`, refuses shared
intermediates before compilation, and refuses an unrelated compiler diagnostic,
including filenames and source snippets containing the expected code. Failed
diagnostic qualification retains a report with null summaries and no timed samples.
These timings validate the runner;
they provide no claim about compiler acceleration. Unit tests cover numerical
known cases, ties/nulls, symmetry, bounded intervals, deterministic bootstrap and
invalid inputs. Run only with the parent-supplied produced toolchain:

```text
LUSTR_BENCH_TEST_BLUEFE_ROOT=<toolchain> \
LUSTR_BENCH_TEST_OUTPUT=<new-test-output> \
<toolchain>/bin/lustr --bluefe-root <toolchain> test --locked \
  --manifest-path crates/lustr-bench/Cargo.toml --target-dir <build-output> -j 2
```

Set `LUSTR_BENCH_TEST_STOCK_CARGO` to an absolute second Cargo executable to
run the additional real-version metadata regression. Both lanes use the same
explicit produced Rust compiler. The test accepts the observed unset optional
hint, then adds a real non-null hint and requires a failed report with null
summaries and no timed samples. Its fixture timings validate machinery only.

Campaign workloads can include bluefin `ferrofin-model` at
`b7d124a22d49bacf19f6c3c940333f508d647fb6`, t3code-ls core/storage at
`b16018167df26a18ee75d3d2b8f4bf2fcd64ce92`, and gitr at
`a183e205ec21e1b916e3763ba85917f2842aeae5`. Parent supplies snapshots, real
project-specific tests/rejection fixtures and exact tool identities. These large
workloads have not been benchmarked by the runner implementation task. Existing
Python benchmark sources, raw negative results and reports remain historical
evidence and are not replaced or relabeled.

After delivery, preserve the current executable, source hashes, exact spec,
reports and small logs. Remove only owned disposable target/cache/staging trees
after checking active processes, descriptors, mounts and the Cargo build lock.
Record actual available-space before/after; shared concurrent work can confound
the filesystem delta. Do not move build junk to media storage or reserved disks.

## Optional Linux CPU and peak RSS

A spec may set `"resource_mode": "linux-waited"`; omission or `"disabled"`
preserves the direct-command protocol. Enable the same mode for every lane before
collecting a campaign. There are no resource thresholds or automatic gate advances.
Historical wall-time datasets remain unchanged; they have no resource observations.

Each command gets a fresh reexecution of the hashed runner as a collector. It
waits for the payload and calls the pinned nix safe `getrusage(RUSAGE_CHILDREN)`
wrapper. Owned Rust forbids unsafe code; nix/libc and Linux remain upstream OS
compatibility dependencies, not evidence of whole-toolchain memory safety.
The request binds raw argument/environment bytes, working directory, unique report
path and collector executable hash. The fresh report binds the request hash,
collector hash/PID, protocol scope, raw wait status and measurements. Protocol v2
requires a zero collector exit after report publication. The report preserves
the payload exit or signal separately, including SIGPIPE and inherited blocked
signals, without re-raising it in the collector. A nonzero or signalled collector
exit invalidates the report. The runner still rejects a signalled payload and
uses the payload exit code when checking expected failures. Missing, malformed,
stale, mismatched or interrupted reports fail qualification with null summaries.
Requests and raw reports remain available beside byte-preserving stdout/stderr.
The payload receives exactly the direct-command environment and null stdin;
collector control uses arguments and its own stdin lease, not environment markers.
The internal entry point requires process-group leadership before starting the
lease thread; ordinary runner invocation creates that group. Lease closure kills
the checked group. This precondition prevents inherited-group misuse; it does
not prove universal group ownership. Existing process-group limitations
still apply: deliberately detached sessions are not a whole-system tracing scope.

Linux defines user/system CPU in seconds plus microseconds; records use integer
microseconds. CPU is the sum for terminated, waited children, including deeper
descendants only when every intervening parent waited. Unwaited or detached
work is outside this scope. Linux peak RSS is in KiB and is the largest waited
process peak, **not simultaneous aggregate process-tree or cgroup memory**.
These semantics follow [Linux getrusage(2)](https://man7.org/linux/man-pages/man2/getrusage.2.html).
For peaks p_i, this collector reports max_i(p_i); it does not report
max_t(sum_i RSS_i(t)). It cannot satisfy an aggregate RAM gate. Energy is unmeasured.

A long-lived collector cannot recover command peak RSS by subtracting counters:
M_after = max(M_before, M_child). With M_before = 100 MiB and M_child = 20 MiB,
the difference is zero, although the child used 20 MiB. A fresh collector per
command avoids that contamination. Synthetic large-then-small tests exercise it.

Outer wall time includes collector startup, payload execution and report
publication. Parent-side executable hashing and request serialization/publication
happen before that wall interval and are excluded. Payload wall time covers
spawn through wait. Collector user/system
CPU is a separate RUSAGE_SELF observation taken before serialization/publication;
it is a partial overhead observation, excluding the final write and exit.
Overhead is not assumed zero or subtracted from wall samples. Existing paired
wall log-ratio statistics and their assumptions are unchanged. Resource values
are raw observations, with no resource log-ratio inference or universal compiler
speed claim; any later resource analysis must preregister scope, thresholds,
zero handling and paired-sample assumptions independently.
