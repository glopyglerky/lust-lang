# Unadmitted native cache experiment

This experiment evaluates a necessary cache correctness repair against unchanged
Lust. It does not admit a source divergence or build a distributable package.
The primary checkout and its complete protected native source remain pristine.
Only an explicitly named isolated candidate worktree receives `candidate.patch`.

The pristine whole-request cache can replay stale values or diagnostics when
comments or whitespace separate an external-input macro name from `!`. A direct
`option_env /* gap */ !` reproduction returns `None` after its environment input
becomes `first` and then `second`.

The candidate replaces five raw substring scans with one conservative scan. A
bounded trivia lookahead avoids lexing ordinary `env` variables twice. When a
possible spaced invocation remains, Lust's own lexer checks the real tokens.
The outer scan never skips an apparent comment, which may actually occur inside
a string. Existing contiguous positives remain positive.

## Run

Use Ubuntu 24.04 x86_64 with Rust 1.99.1, Python 3, Git and C build tools:

```sh
python3 tools/experiments/cache-guard/run.py --work /absolute/temporary/cache-experiment
```

The work path must be outside the primary checkout and must not already exist.
`run.py` verifies every input against the preregistered hashes in `plan.json`,
creates two worktrees at the exact base revision, and patches only the candidate.
It builds TinyCC once and both compilers sequentially using the same release
profile and bootstrap compiler. It verifies source identities before and after.

The correctness checks compare actual native execution against rustc across
external-input changes, comments, nested comments, raw strings, generated macros,
cache-preserving lookalikes, panic recovery and destructor order. Separate owned
daemons check warm external-input changes and are terminated by the test.
Existing native qualification records valid support, invalid rejection and the
remaining coverage gaps independently for both compilers.

The timing experiment uses 40 balanced pairs for each of 14 cases. Source bytes,
flags, cache state and correct outputs match between lanes. Cases include fresh
compilation, request-cache hits, body and signature edits, invalid edits, early
macro matches, the former false-positive class, a long-comment stress case and
the real upstream logscan program. Wrong cached baseline answers never qualify
as equivalent performance work. Raw samples and outcomes are preserved.

A completed workflow only means the experiment finished. `result.json` reports
whether the preregistered speed rule passed and lists confirmed regressions.
Even a passing experiment records `admitted: false` and `packaged: false`.
Admission still requires review of all results, an explicit scoped decision and
source-guard support. Do not copy this patch into a product or bypass the guard
based merely on a green workflow.

The earlier local simultaneous release builds were killed by SIGKILL. No
candidate timing resulted from those attempts. CI builds sequentially to avoid
that contention. These records do not establish a universal speed claim.
