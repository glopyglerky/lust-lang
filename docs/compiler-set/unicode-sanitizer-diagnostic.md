# Unicode sanitizer timeout diagnostic

This branch investigates the recurring `unicode_props_hash` sanitizer runtime
timeout at lustr `82d77539680fec9cf822ca8231d1abb985069c48`.
It does not modify `compiler/native`, fix the compiler, or replace normal CI.

The source log calls it a "sanitized build" timeout. The failing operation is
the generated ASan/UBSan executable, limited to ten seconds. GCC compilation
has a separate twenty-second O1 limit and an O0 fallback.

## Runs and evidence

The workflow builds unchanged Lust with the existing CI Rust 1.99.0 release
profile. It checks the pinned source identity before and after the diagnostic.

1. Run the unchanged upstream differential script for the exact fixture with
   one worker, then for the `unicode_` subset with its default `nproc` pool.
   Both use RUN_TIMEOUT=10, CASE_TIMEOUT=30 and SAN_CC_LIMIT=20. The rustc
   result cache is disabled. Inherited alternate test modes are cleared.
2. Reproduce the fixture independently using the same oracle, plain Lust,
   emitted-C, GCC sanitizer flags and O0 fallback rule. Retain the emitted C,
   objects, binary, exact commands, output, status, wall time, GNU time CPU/RSS
   reports, source/tool identities and runner resource limits.
3. Run three pairs of isolated execution and `nproc` simultaneous copies.
   Alternate their order and use a start barrier for concurrent copies.
4. Save the ten-second results before any optional sixty-second isolated run.
   The longer run is diagnostic only. A ten-second failure remains a failure.

Artifacts from the reproduction are explicitly separate from the unchanged
upstream script, which deletes its own temporary directory. No source wrapper,
generated overlay, compiler patch or changed fixture is used.

The `unicode_` subset does not reproduce full-suite scheduling. Concurrent
copies test executable contention, not the real pool's mixture of compilation,
execution and object-cache locking. A pass cannot rule out those effects.
Any O0 fallback is recorded because it changes the performance specimen.

The recorder has an outer session-cleanup watchdog to bound a stalled
investigation. Such a timeout is an incomplete diagnostic, never a passing
qualification. It kills and verifies the launched session, including GNU timeout's
separate process groups, then aborts before any later phase. GNU timeout retains the upstream ten-second TERM/124 behavior.
The O0 fallback has no inner compile limit, matching upstream.

## Scope and execution

The workflow runs only on pushes to
`diagnostics/unicode-sanitizer-82d7753`, on GitHub's Ubuntu 24.04 runner.
It uses read-only repository permissions and retains artifacts for fourteen
days. It does not rerun or cancel existing CI, open a PR that triggers the full
suite, merge into `next`, publish a release, or use the user's infrastructure.

The ordinary CI source and its threshold remain unchanged. This diagnostic
does not establish native full-Rust support or resolve the separate macOS link,
environment-read invalidation or Windows fixture failures.

Recorder self-test:

```sh
python3 tools/diagnose-unicode-sanitizer.py --self-test
```

Full focused investigation, after building the existing release compiler:

```sh
python3 tools/diagnose-unicode-sanitizer.py /path/to/new/evidence-directory
```

Sources:
- [Current failure](https://github.com/glopyglerky/lustr/actions/runs/42740215832/job/129579127202)
- [Earlier occurrence](https://github.com/glopyglerky/lustr/actions/runs/42526019095/job/128869758754)
- [Unchanged sanitizer runner](https://github.com/glopyglerky/lustr/blob/82d77539680fec9cf822ca8231d1abb985069c48/compiler/native/scripts/difftest.sh)
