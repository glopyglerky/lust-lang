# Cache lookup must not invoke Cargo

The native compiler receives Cargo's effective arguments and environment.
Its request cache must not discover an enclosing manifest and start another
Cargo invocation while storing or retrieving that completed request.

The initial cache candidate appended a suffix to the native package identity.
Upstream cache-key construction still recognized that identity as a package,
searched for its manifest and refreshed its build script. A fake Cargo command
recorded an invocation during both the first store and the warm lookup, even
though the native compilation itself succeeded. The original reproduction is
retained in [original-probe.json.gz](original-probe.json.gz).

The fix uses upstream's synthetic `file:` namespace only for the cache
surrogate. The digest still binds the original request, native arguments and
environment. Compilation retains the original session and ABI. Requests that
explicitly carry a manifest do not enter this bounded cache.

## Regression evidence

The expanded functional check puts a manifest and a deliberately invalid build
script beside the source. A logging Cargo sentinel must remain untouched.
After the first successful compile, the test disables the private TinyCC helper
and removes the output, proving that the next success comes from actual cache
reuse. Changing the parent manifest, build script and Cargo configuration with
the same effective compiler request must still avoid Cargo re-entry.

| Diagnostic binary | Result | Checks reached | Commands |
| --- | --- | ---: | ---: |
| Initial cache, `734d5086` | Failed at the first unexpected Cargo invocation | 73 | 57 |
| Fixed namespace, `9447fad2` | Passed | 84 | 64 |

Both runs used the same expanded test, SHA-256
`c97e47b5fa102d7aacea8db0c75c3b087b68d9549298ea3576cc91ef3b244acb`.
The [red report](red-report.json.gz), [green report](green-report.json.gz),
[exact source change](source-change.diff.gz) and
[diagnostic build log](diagnostic-build.log.gz) identify the measured code.
All command-output hashes were checked against [logs.tar.gz](logs.tar.gz).
[manifest.json](manifest.json) binds the original and compressed bytes and the
complete compiler hashes.

These are optimization-level-1 functional diagnostics. They establish this
regression fix, not release performance, complete upstream compatibility or
full Rust support. The preceding [release candidate](../cache-candidate/README.md)
predates the fix and cannot measure it.
