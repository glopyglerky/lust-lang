# lust speed on real ts-rust code

The promise: **a warm `lust check` after a one-function edit is 100x faster than
`cargo check` (rustc 1.99, incremental) on the same source.** The source is a
pinned slice of ts-rust, not generated code. Cold check, build, and restart are
reported in their own rows; they are not the promise.

## Method

- Corpus: `bench/slices/<name>`, real ts-rust source pinned at a commit, minus
  the items lust cannot take yet (see `bench/slices/README.md`). lust and rustc
  both accept it, and `lust build` builds it.
- One source tree for both compilers. Every row runs lust, then cargo, on the
  same files.
- lust runs through a warm `lust daemon` (the client is the `lust` command).
  Cold rows use a one-shot process with an empty cache.
- rustc runs through cargo in two configs, each with its own target dir:
  `default` (stable dev profile, incremental on) and `threads8`
  (`RUSTC_BOOTSTRAP=1`, `-Z threads=8`). Both use `debuginfo = 0`. One config
  is picked per row by its median. The table gives the ratio for the median
  and for p95, both from that config. The 100x goal is tracked at the median,
  at p95, and at the worst request.
- For the daemon first check, no-op, and cached restart rows, the rustc column
  is a no-op `cargo check` (nothing changed since its last check).
- Body edits: 120 edits that alternate across 24 fns in 24 different files.
  Each edit inserts or changes one `let` statement in one fn body.
- Signature, type, and impl edits are in the slice's `edits.json`. Each is
  applied and reverted, 5 cycles. The error edit puts a type error in a body
  (both compilers must reject it); the fix edit removes it.
- Exactness: on every source state, lust and cargo agree on pass or fail. Every
  10th state and every non-body edit, a cold one-shot `lust check` must give
  the same stdout, stderr, and exit code as the warm daemon. A mismatch stops
  the run.
- Wall time of the whole command (client or cargo process), median, p95, and
  worst. Phase times come from `lust check --timings`. A phase median runs
  over all requests of the row. A phase a request did not emit counts as
  zero. A separate line gives the median over only the requests that did
  emit the phase, with its count. The result JSON (`requests`) saves one
  record per lust request: edit id, target fn, wall time, every phase, and
  the `lust-visits:` counters. It also saves the body-patch and kept-program
  (`kept=`) hit or miss and the miss reason. Peak RSS: the daemon's VmHWM;
  for cargo, the largest process in its tree.
- Host: one boat sandbox per run (16 vCPU, nothing else running).
- The slice's empty `main` calls `bench/slices/driver.rs` in the benchmark
  copy: it calls kept ts-rust tspath fns and prints the results. After every
  build, the lust binary and each rustc binary run, and stdout, stderr, and exit
  code must match. That gives the run and build+run rows. The run row is tiny
  work; it is there for exactness, not speed.

Reproduce:

```sh
cargo build --release
python3 scripts/scale-gate.py --slice bench/slices/goport-util --json gu.json
python3 scripts/speed_slice.py --render docs/speed.md all4.json all.json full.json nochecker.json lsp.json gu.json small.json
```

The quick gate row: `scripts/speed-row.sh` (small slice, 20 body edits).

## Where we are (2026-10-15, lust next 4f4c96f84)

The promise row is **body edit check**. On the biggest slice, `tsrust-all4`
(all of ts_goport that lust takes today, 165k lines), the median is
**644.2x** (lust 3.7 ms, rustc 2,371 ms) and the p95 is **242.4x** (12 ms).
The worst request is 123 ms. The p95 passes 100x on all four slices from
107k lines up, and the median passes 100x from 38k lines up.

| Slice | lines | warm body edit check, lust / rustc | ratio median | ratio p95 | p95 lust | worst lust | patch hit/miss | kept hit/miss | no-op | cold check ratio | cold build+run ratio |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| tsrust-all4 | 165,246 | 3.7 / 2,371 ms | **644.2x** | **242.4x** | 12 ms | 123 ms | 120/0 | 115/5 | 1.2 ms | 2.0x | 1.7x |
| tsrust-all | 160,107 | 3.3 / 2,038 ms | **625.1x** | **345.7x** | 7 ms | 102 ms | 120/0 | 115/5 | 1.1 ms | 1.9x | 1.7x |
| tsrust-full | 150,189 | 4.1 / 2,113 ms | **519.1x** | **187.3x** | 15 ms | 143 ms | 120/0 | 115/5 | 2.9 ms | 2.1x | 1.8x |
| tsrust-nochecker | 107,126 | 3.1 / 1,145 ms | **370.5x** | **199.8x** | 7 ms | 83 ms | 120/0 | 115/5 | 1.7 ms | 2.4x | 1.9x |
| goport-util-lsp | 53,960 | 3.2 / 774 ms | **244.9x** | **82.6x** | 12 ms | 65 ms | 120/0 | 115/5 | 1.2 ms | 4.7x | 3.0x |
| goport-util | 37,939 | 2.3 / 274 ms | **119.2x** | **9.2x** | 33 ms | 107 ms | 115/5 | 110/10 | 1.1 ms | 2.9x | 1.8x |
| goport-util-small | 8,379 | 1.8 / 85 ms | **46.4x** | **39.2x** | 3 ms | 4 ms | 120/0 | 120/0 | 1.0 ms | 2.5x | 1.4x |

Line counts include the 42-line driver. Each slice ran alone on its own
sandbox (W-BENCH4), on the same tree. Absolute times move by about 25%
between sandboxes: compare ratios, not milliseconds. W-BENCH3 (4c9f1cab1 +
3dd33ca21) had p95 ratios of 162.5x, 22.0x, 2.1x, 7.3x, 2.2x, and 2.1x on
the six older slices.

- Patch misses fell from 105 to 5 over the six older slices (W-PATCH-MISS
  and W-PATCH-MISS2: cfg-disabled fns, fns that hold items or macros).
  `lib_parse_snapshot.rs` `mode` (1,267 ms on tsrust-full before) now hits.
- One patch miss is left: `fswatch/fanotify_linux.rs`
  `make_fanotify_handle_key` (`signature_nodes`, 100 to 107 ms, 5 per run on
  goport-util only, the one slice that picks it as a target). Its kept miss
  reason is reported as `none`.
- One kept-program miss is left: `gostd/unicode_tables.rs` `map_get`
  (`fn_class`, a full collect: 87 to 123 ms on the tsrust slices). It is the
  worst request on every slice from 38k lines up, and the whole tail above
  p95 there.
- On a patch hit with a kept program, a warm body edit costs 2 to 4 ms at
  the median and at most 33 ms.
- Signature, type, and impl edits are 1.5x to 1.7x on tsrust-all4: they
  cost close to a cold check. Body edit build is 17.4x (p95 9.3x).
- The daemon's peak RSS on tsrust-all4 is 3.3 GB; cargo's largest process
  is 1.1 GB (threads8).
- Cold check is 1.9x to 4.7x. rustc with `-Z threads=8` is fast on these
  slices.

## Results

<!-- speed-table:start -->
### `tsrust-all4`: 165,246 lines (102,440 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 1394 ms | 1428 | 1429 | 5 | 2777 ms | 2892 ms (threads8) | 2.0x | 2.0x |
| cold build | 1734 ms | 1798 | 1802 | 5 | 2933 ms | 3075 ms (threads8) | 1.7x | 1.7x |
| run (built program) | 2 ms | 2 | 3 | 17 | 1 ms | 1 ms (threads8) | 0.33x | 0.39x |
| cold build+run | 1735 ms | 1800 | 1803 | 5 | 2933 ms | 3075 ms (threads8) | 1.7x | 1.7x |
| daemon first check (new session, empty cache) | 1632 ms | 1632 | 1632 | 1 | 11 ms | 11 ms (threads8) | 0.01x | 0.01x |
| no-op check (warm daemon) | 1 ms | 1 | 1 | 5 | 10 ms | 11 ms (threads8) | 8.9x | 7.7x |
| cached restart check (new daemon, disk cache) | 1568 ms | 1599 | 1601 | 5 | 12 ms | 15 ms (threads8) | 0.01x | 0.01x |
| body edit check (one fn) | 4 ms | 12 | 123 | 120 | 2371 ms | 2816 ms (threads8) | **644.2x** | **242.4x** |
| body edit build (one fn) | 148 ms | 349 | 484 | 12 | 2576 ms | 3262 ms (threads8) | 17.4x | 9.3x |
| body edit build+run (one fn) | 150 ms | 351 | 486 | 12 | 2577 ms | 3263 ms (threads8) | 17.2x | 9.3x |
| signature edit check | 1674 ms | 1866 | 1867 | 10 | 2481 ms | 2812 ms (threads8) | 1.5x | 1.5x |
| type edit check | 1608 ms | 1677 | 1686 | 10 | 2556 ms | 2774 ms (threads8) | 1.6x | 1.7x |
| impl edit check | 1635 ms | 1886 | 1904 | 10 | 2608 ms | 3095 ms (threads8) | 1.6x | 1.6x |
| error edit check (both reject) | 169 ms | 183 | 186 | 5 | 1925 ms | 2132 ms (threads8) | 11.4x | 11.6x |
| fix edit check | 180 ms | 217 | 220 | 5 | 2245 ms | 2344 ms (default) | 12.5x | 10.8x |

Cold build: 18.6 MB of C, 38 units, link 13 ms.
Peak RSS: lust cold check 935 MB, lust daemon 3303 MB (VmHWM over the run), cargo cold check 876 MB (default), 1114 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 115 hit, 5 miss (fn_class x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 2, collect 1, check changed fns (par) 0, route 0, read 0.

### `tsrust-all`: 160,107 lines (98,983 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 1136 ms | 1152 | 1153 | 5 | 2200 ms | 2238 ms (threads8) | 1.9x | 1.9x |
| cold build | 1372 ms | 1391 | 1394 | 5 | 2386 ms | 2395 ms (threads8) | 1.7x | 1.7x |
| run (built program) | 1 ms | 2 | 2 | 17 | 0 ms | 1 ms (threads8) | 0.31x | 0.33x |
| cold build+run | 1372 ms | 1392 | 1395 | 5 | 2387 ms | 2396 ms (threads8) | 1.7x | 1.7x |
| daemon first check (new session, empty cache) | 1355 ms | 1355 | 1355 | 1 | 11 ms | 11 ms (threads8) | 0.01x | 0.01x |
| no-op check (warm daemon) | 1 ms | 1 | 1 | 5 | 10 ms | 12 ms (default) | 8.5x | 9.0x |
| cached restart check (new daemon, disk cache) | 1285 ms | 1305 | 1309 | 5 | 11 ms | 11 ms (threads8) | 0.01x | 0.01x |
| body edit check (one fn) | 3 ms | 7 | 102 | 120 | 2038 ms | 2333 ms (default) | **625.1x** | **345.7x** |
| body edit build (one fn) | 123 ms | 261 | 394 | 12 | 2200 ms | 2498 ms (threads8) | 17.9x | 9.6x |
| body edit build+run (one fn) | 124 ms | 263 | 396 | 12 | 2201 ms | 2499 ms (threads8) | 17.7x | 9.5x |
| signature edit check | 1296 ms | 1449 | 1530 | 10 | 1991 ms | 2267 ms (default) | 1.5x | 1.6x |
| type edit check | 1268 ms | 1495 | 1600 | 10 | 2087 ms | 2185 ms (threads8) | 1.6x | 1.5x |
| impl edit check | 1262 ms | 1365 | 1368 | 10 | 2017 ms | 2123 ms (threads8) | 1.6x | 1.6x |
| error edit check (both reject) | 133 ms | 136 | 136 | 5 | 1599 ms | 1769 ms (threads8) | 12.0x | 13.0x |
| fix edit check | 133 ms | 138 | 139 | 5 | 1920 ms | 2077 ms (default) | 14.5x | 15.0x |

Cold build: 17.0 MB of C, 38 units, link 10 ms.
Peak RSS: lust cold check 922 MB, lust daemon 3159 MB (VmHWM over the run), cargo cold check 841 MB (default), 1069 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 115 hit, 5 miss (fn_class x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 1, collect 1, check changed fns (par) 0, route 0, read 0.

### `tsrust-full`: 150,189 lines (91,879 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 1521 ms | 1575 | 1587 | 5 | 3171 ms | 3465 ms (threads8) | 2.1x | 2.2x |
| cold build | 1879 ms | 2081 | 2128 | 5 | 3430 ms | 3949 ms (threads8) | 1.8x | 1.9x |
| run (built program) | 2 ms | 2 | 3 | 17 | 1 ms | 1 ms (threads8) | 0.4x | 0.54x |
| cold build+run | 1880 ms | 2083 | 2130 | 5 | 3431 ms | 3950 ms (threads8) | 1.8x | 1.9x |
| daemon first check (new session, empty cache) | 2134 ms | 2134 | 2134 | 1 | 15 ms | 15 ms (threads8) | 0.01x | 0.01x |
| no-op check (warm daemon) | 3 ms | 3 | 3 | 5 | 17 ms | 20 ms (threads8) | 5.9x | 6.5x |
| cached restart check (new daemon, disk cache) | 1575 ms | 1634 | 1645 | 5 | 18 ms | 21 ms (threads8) | 0.01x | 0.01x |
| body edit check (one fn) | 4 ms | 15 | 143 | 120 | 2113 ms | 2886 ms (threads8) | **519.1x** | **187.3x** |
| body edit build (one fn) | 137 ms | 414 | 697 | 12 | 2304 ms | 3020 ms (threads8) | 16.9x | 7.3x |
| body edit build+run (one fn) | 139 ms | 416 | 699 | 12 | 2305 ms | 3020 ms (threads8) | 16.6x | 7.3x |
| signature edit check | 1206 ms | 1316 | 1347 | 10 | 1830 ms | 2054 ms (default) | 1.5x | 1.6x |
| type edit check | 1224 ms | 1383 | 1452 | 10 | 2065 ms | 2188 ms (threads8) | 1.7x | 1.6x |
| impl edit check | 1271 ms | 1365 | 1377 | 10 | 1953 ms | 2239 ms (threads8) | 1.5x | 1.6x |
| error edit check (both reject) | 124 ms | 159 | 163 | 5 | 1503 ms | 1624 ms (threads8) | 12.1x | 10.2x |
| fix edit check | 118 ms | 156 | 163 | 5 | 1758 ms | 2029 ms (threads8) | 14.9x | 13.0x |

Cold build: 15.4 MB of C, 38 units, link 16 ms.
Peak RSS: lust cold check 767 MB, lust daemon 2769 MB (VmHWM over the run), cargo cold check 748 MB (default), 964 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 115 hit, 5 miss (fn_class x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 2, collect 1, check changed fns (par) 0, route 0, read 0.

### `tsrust-nochecker`: 107,126 lines (67,850 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 607 ms | 638 | 646 | 5 | 1429 ms | 1556 ms (threads8) | 2.4x | 2.4x |
| cold build | 802 ms | 871 | 874 | 5 | 1555 ms | 1768 ms (threads8) | 1.9x | 2.0x |
| run (built program) | 1 ms | 2 | 2 | 17 | 0 ms | 1 ms (threads8) | 0.34x | 0.34x |
| cold build+run | 803 ms | 872 | 875 | 5 | 1555 ms | 1768 ms (threads8) | 1.9x | 2.0x |
| daemon first check (new session, empty cache) | 733 ms | 733 | 733 | 1 | 11 ms | 11 ms (threads8) | 0.01x | 0.01x |
| no-op check (warm daemon) | 2 ms | 2 | 2 | 5 | 11 ms | 14 ms (threads8) | 6.4x | 7.0x |
| cached restart check (new daemon, disk cache) | 640 ms | 721 | 733 | 5 | 12 ms | 14 ms (threads8) | 0.02x | 0.02x |
| body edit check (one fn) | 3 ms | 7 | 83 | 120 | 1145 ms | 1305 ms (default) | **370.5x** | **199.8x** |
| body edit build (one fn) | 88 ms | 205 | 328 | 12 | 1263 ms | 1397 ms (threads8) | 14.4x | 6.8x |
| body edit build+run (one fn) | 89 ms | 207 | 330 | 12 | 1263 ms | 1397 ms (threads8) | 14.1x | 6.8x |
| signature edit check | 564 ms | 786 | 912 | 10 | 1084 ms | 1476 ms (default) | 1.9x | 1.9x |
| type edit check | 572 ms | 636 | 663 | 10 | 1180 ms | 1416 ms (threads8) | 2.1x | 2.2x |
| impl edit check | 566 ms | 615 | 624 | 10 | 1148 ms | 1366 ms (threads8) | 2.0x | 2.2x |
| error edit check (both reject) | 75 ms | 77 | 78 | 5 | 864 ms | 907 ms (threads8) | 11.5x | 11.7x |
| fix edit check | 78 ms | 78 | 78 | 5 | 1007 ms | 1136 ms (default) | 12.9x | 14.5x |

Cold build: 12.6 MB of C, 38 units, link 8 ms.
Peak RSS: lust cold check 413 MB, lust daemon 1466 MB (VmHWM over the run), cargo cold check 538 MB (default), 696 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 115 hit, 5 miss (fn_class x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 1, collect 1, check changed fns (par) 0, route 0, read 0.

### `goport-util-lsp`: 53,960 lines (36,890 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 175 ms | 184 | 184 | 5 | 820 ms | 969 ms (threads8) | 4.7x | 5.3x |
| cold build | 311 ms | 354 | 356 | 5 | 932 ms | 1212 ms (threads8) | 3.0x | 3.4x |
| run (built program) | 2 ms | 2 | 3 | 17 | 1 ms | 1 ms (threads8) | 0.44x | 0.41x |
| cold build+run | 314 ms | 355 | 357 | 5 | 932 ms | 1213 ms (threads8) | 3.0x | 3.4x |
| daemon first check (new session, empty cache) | 307 ms | 307 | 307 | 1 | 11 ms | 11 ms (threads8) | 0.04x | 0.04x |
| no-op check (warm daemon) | 1 ms | 2 | 2 | 5 | 11 ms | 11 ms (default) | 8.8x | 7.3x |
| cached restart check (new daemon, disk cache) | 238 ms | 246 | 248 | 5 | 12 ms | 15 ms (threads8) | 0.05x | 0.06x |
| body edit check (one fn) | 3 ms | 12 | 65 | 120 | 774 ms | 981 ms (default) | **244.9x** | **82.6x** |
| body edit build (one fn) | 73 ms | 178 | 265 | 12 | 882 ms | 1086 ms (threads8) | 12.0x | 6.1x |
| body edit build+run (one fn) | 75 ms | 179 | 266 | 12 | 883 ms | 1087 ms (threads8) | 11.7x | 6.1x |
| signature edit check | 159 ms | 204 | 212 | 10 | 689 ms | 895 ms (default) | 4.3x | 4.4x |
| type edit check | 192 ms | 236 | 239 | 10 | 834 ms | 1120 ms (threads8) | 4.3x | 4.8x |
| impl edit check | 216 ms | 269 | 272 | 10 | 823 ms | 972 ms (threads8) | 3.8x | 3.6x |
| error edit check (both reject) | 65 ms | 85 | 85 | 5 | 528 ms | 703 ms (threads8) | 8.1x | 8.3x |
| fix edit check | 55 ms | 76 | 77 | 5 | 632 ms | 819 ms (threads8) | 11.5x | 10.7x |

Cold build: 9.2 MB of C, 39 units, link 7 ms.
Peak RSS: lust cold check 170 MB, lust daemon 710 MB (VmHWM over the run), cargo cold check 387 MB (default), 514 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 115 hit, 5 miss (fn_class x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 1, collect 1, check changed fns (par) 0, read 0, route 0.

### `goport-util`: 37,939 lines (28,695 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 115 ms | 141 | 145 | 5 | 327 ms | 448 ms (threads8) | 2.9x | 3.2x |
| cold build | 223 ms | 245 | 246 | 5 | 406 ms | 490 ms (threads8) | 1.8x | 2.0x |
| run (built program) | 1 ms | 2 | 2 | 17 | 1 ms | 1 ms (threads8) | 0.45x | 0.44x |
| cold build+run | 224 ms | 246 | 247 | 5 | 407 ms | 491 ms (threads8) | 1.8x | 2.0x |
| daemon first check (new session, empty cache) | 216 ms | 216 | 216 | 1 | 10 ms | 10 ms (threads8) | 0.05x | 0.05x |
| no-op check (warm daemon) | 1 ms | 1 | 1 | 5 | 9 ms | 10 ms (threads8) | 8.7x | 7.4x |
| cached restart check (new daemon, disk cache) | 137 ms | 147 | 148 | 5 | 10 ms | 11 ms (threads8) | 0.07x | 0.08x |
| body edit check (one fn) | 2 ms | 33 | 107 | 120 | 274 ms | 303 ms (default) | **119.2x** | **9.2x** |
| body edit build (one fn) | 47 ms | 92 | 142 | 12 | 337 ms | 392 ms (default) | 7.2x | 4.3x |
| body edit build+run (one fn) | 48 ms | 93 | 143 | 12 | 337 ms | 393 ms (default) | 7.0x | 4.2x |
| signature edit check | 107 ms | 131 | 132 | 10 | 275 ms | 315 ms (default) | 2.6x | 2.4x |
| type edit check | 119 ms | 134 | 138 | 10 | 342 ms | 382 ms (threads8) | 2.9x | 2.9x |
| impl edit check | 128 ms | 143 | 147 | 10 | 314 ms | 364 ms (threads8) | 2.5x | 2.5x |
| error edit check (both reject) | 33 ms | 40 | 41 | 5 | 234 ms | 293 ms (default) | 7.2x | 7.3x |
| fix edit check | 33 ms | 47 | 50 | 5 | 280 ms | 329 ms (threads8) | 8.5x | 7.0x |

Cold build: 7.8 MB of C, 39 units, link 5 ms.
Peak RSS: lust cold check 160 MB, lust daemon 668 MB (VmHWM over the run), cargo cold check 259 MB (default), 354 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 115 hit, 5 miss (signature_nodes x5). kept program: 110 hit, 10 miss (fn_class x5, none x5).

lust phases, body edit check (median ms over all requests, absent = 0): body patch 1, collect 0, check changed fns (par) 0, read 0, route 0.

lust phases when present only (fallback requests, median ms): resolve: expand+parse 30 (5/120 requests), resolve: discovery 15 (5/120 requests), resolve: strip+register 6 (5/120 requests), resolve: chunk scan+concat 6 (5/120 requests), resolve: other 4 (5/120 requests).

### `goport-util-small`: 8,379 lines (5,014 code)

ts-rust `b11b618ae`, lust `4f4c96f84`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU. 171 source states; lust and cargo agree on pass or fail on every one.

| Row | lust median | p95 | worst | n | rustc median | rustc p95 (config) | Ratio median | Ratio p95 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| cold check | 55 ms | 58 | 58 | 5 | 138 ms | 145 ms (threads8) | 2.5x | 2.5x |
| cold build | 124 ms | 132 | 133 | 5 | 171 ms | 178 ms (threads8) | 1.4x | 1.4x |
| run (built program) | 1 ms | 1 | 1 | 17 | 1 ms | 1 ms (threads8) | 0.59x | 0.51x |
| cold build+run | 125 ms | 133 | 134 | 5 | 171 ms | 179 ms (threads8) | 1.4x | 1.3x |
| daemon first check (new session, empty cache) | 63 ms | 63 | 63 | 1 | 10 ms | 10 ms (threads8) | 0.16x | 0.16x |
| no-op check (warm daemon) | 1 ms | 1 | 1 | 5 | 9 ms | 10 ms (default) | 8.7x | 8.0x |
| cached restart check (new daemon, disk cache) | 74 ms | 76 | 77 | 5 | 10 ms | 12 ms (threads8) | 0.14x | 0.15x |
| body edit check (one fn) | 2 ms | 3 | 4 | 120 | 85 ms | 105 ms (default) | **46.4x** | **39.2x** |
| body edit build (one fn) | 23 ms | 63 | 104 | 12 | 126 ms | 143 ms (default) | 5.5x | 2.3x |
| body edit build+run (one fn) | 24 ms | 64 | 105 | 12 | 127 ms | 144 ms (default) | 5.3x | 2.3x |
| signature edit check | 41 ms | 48 | 48 | 10 | 73 ms | 83 ms (default) | 1.8x | 1.7x |
| type edit check | 44 ms | 54 | 56 | 10 | 98 ms | 118 ms (threads8) | 2.2x | 2.2x |
| impl edit check | 52 ms | 57 | 57 | 10 | 99 ms | 110 ms (threads8) | 1.9x | 1.9x |
| error edit check (both reject) | 3 ms | 4 | 4 | 5 | 75 ms | 80 ms (default) | 23.9x | 22.6x |
| fix edit check | 2 ms | 3 | 3 | 5 | 72 ms | 79 ms (default) | 41.9x | 30.5x |

Cold build: 3.3 MB of C, 23 units, link 3 ms.
Peak RSS: lust cold check 71 MB, lust daemon 411 MB (VmHWM over the run), cargo cold check 147 MB (default), 212 MB (threads8) (largest process in the tree).

Body edit check paths: body patch: 120 hit, 0 miss. kept program: 120 hit, 0 miss.

lust phases, body edit check (median ms over all requests, absent = 0): body patch 0, collect 0, check changed fns (par) 0, read 0, route 0.

<!-- speed-table:end -->

## Edit kinds on real ts-rust targets

The slice rows above edit fn bodies only. `scripts/tsedit.py` measures eight
kinds of one-fn edits on the largest real code that checks today: the census
clean fork of each lib target (`scripts/tsrust_deep.py --rustc --save-clean`:
items removed until both lust and `cargo check` accept the target), and the
slices. Each probe edits one fn or item, then goes back to the base text.

- `body`: a `let` in one fn body. `infer`: a `let` whose element type comes
  from a later use, then a use that changes that type. `sig`: a real fn gets
  `where u32: Copy`. `sigx`: a crate-root fn called from fns in other modules
  changes its param and return types (callers still fit through `.into()`).
  `oblig`: a body asks a trait of a real type that it derives. `derive`: one
  trait goes out of a real `#[derive(..)]`. `generic`: a body edit in a
  generic fn that other files name. `rpit`: a fn that returns `impl Trait`
  gets a body edit, then a `+ Sized` bound.
- Each kind also has reject states (`let x: u32 = "1"`, `where String: Copy`,
  one more parameter, a trait the type lacks, `Option<T>::cloned`,
  `+ Copy`, a return type callers cannot take). On every state the warm
  daemon answer must match rustc's verdict and a cold one-shot lust check
  (exit code, stdout, stderr). A timeout is a hang, never a reject.
- Ratios use the states rustc accepts: `cargo check` of the same target in a
  warm target dir with the same features as lust (`.lust.toml`), over the
  warm daemon request. One boat sandbox per run, release lust.

Reproduce: `scripts/tsrust-full.sh --boat --tsedit` (census, then every lib
target), or `python3 scripts/tsedit.py --slice bench/slices/goport-util`.
`python3 scripts/tsedit.py --render docs/speed.md a.json ...` rewrites the
tables below. The gate's speed row runs one probe per kind on
goport-util-small.

<!-- tsedit-table:start -->
### `ts_goport (clean fork)`: 151,330 lines, 683 files

ts-rust `660832d85`, lust `c938c0f9b`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU, 8 probes per kind, 200 states. Daemon first check 4226 ms, no-op 3.91 ms.

| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| body | 16 | 7.9 ms | 1416.1 | 3187 ms | 3592 | **401.3x** | **2.5x** | 8 | 0 | 0 |
| infer | 24 | 246.7 ms | 1405.6 | 3101 ms | 3222 | **12.6x** | **2.3x** | 8 | 0 | 0 |
| sig | 19 | 1958.8 ms | 2603.9 | 3077 ms | 3193 | **1.6x** | **1.2x** | 13 | 0 | 0 |
| sigx | 12 | 1863.6 ms | 2587.9 | 3060 ms | 3317 | **1.6x** | **1.3x** | 4 | 0 | 0 |
| oblig | 16 | 12.2 ms | 1534.5 | 3965 ms | 5170 | **325.2x** | **3.4x** | 8 | 0 | 0 |
| derive | 15 | 3935.9 ms | 7019.8 | 8195 ms | 15403 | **2.1x** | **2.2x** | 1 | 0 | 0 |
| generic | 18 | 9.3 ms | 16.5 | 4158 ms | 4746 | **446.7x** | **288.1x** | 6 | 0 | 0 |
| rpit | 24 | 2457.3 ms | 5710.6 | 4212 ms | 9229 | **1.7x** | **1.6x** | 8 | 0 | 0 |

### `goport_util (clean fork)`: 77,488 lines, 80 files

ts-rust `660832d85`, lust `c938c0f9b`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU, 6 probes per kind, 132 states. Daemon first check 1349 ms, no-op 2.56 ms.

| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| body | 12 | 4.8 ms | 8.7 | 628 ms | 660 | **131.1x** | **76.0x** | 6 | 0 | 0 |
| infer | 18 | 65.8 ms | 77.9 | 636 ms | 685 | **9.7x** | **8.8x** | 6 | 0 | 0 |
| sig | 14 | 189.1 ms | 279.5 | 612 ms | 667 | **3.2x** | **2.4x** | 10 | 0 | 0 |
| sigx | 9 | 229.0 ms | 286.0 | 707 ms | 754 | **3.1x** | **2.6x** | 3 | 0 | 0 |
| oblig | 12 | 4.9 ms | 11.3 | 638 ms | 672 | **129.5x** | **59.3x** | 6 | 0 | 0 |
| derive | 9 | 293.6 ms | 303.9 | 945 ms | 1157 | **3.2x** | **3.8x** | 3 | 0 | 0 |
| generic | 13 | 4.3 ms | 15.8 | 563 ms | 630 | **131.1x** | **39.9x** | 5 | 0 | 0 |
| rpit | 5 | 12.3 ms | 182.1 | 566 ms | 596 | **45.9x** | **3.3x** | 1 | 0 | 0 |

### `goport_lsproto (clean fork)`: 31,500 lines, 17 files

ts-rust `660832d85`, lust `c938c0f9b`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU, 6 probes per kind, 108 states. Daemon first check 1696 ms, no-op 3.33 ms.

| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| body | 12 | 5.0 ms | 12.4 | 1126 ms | 1212 | **224.7x** | **97.6x** | 6 | 0 | 0 |
| infer | 18 | 108.5 ms | 130.1 | 1230 ms | 1315 | **11.3x** | **10.1x** | 6 | 0 | 0 |
| sig | 15 | 291.3 ms | 443.0 | 1115 ms | 1248 | **3.8x** | **2.8x** | 9 | 0 | 0 |
| sigx | 9 | 285.0 ms | 323.5 | 1186 ms | 1289 | **4.2x** | **4.0x** | 3 | 0 | 0 |
| oblig | 12 | 5.3 ms | 9.2 | 1160 ms | 1236 | **217.7x** | **135.0x** | 6 | 0 | 0 |
| derive | 7 | 493.9 ms | 523.3 | 1058 ms | 1650 | **2.1x** | **3.2x** | 5 | 0 | 0 |
| generic | 0 | - | - | - | - | - | - | 0 | 0 | 0 |
| rpit | 0 | - | - | - | - | - | - | 0 | 0 | 0 |

### `tsrust-all4`: 165,987 lines, 783 files

ts-rust `b11b618ae`, lust `c938c0f9b`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU, 6 probes per kind, 150 states. Daemon first check 2414 ms, no-op 4.24 ms.

| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| body | 12 | 7.2 ms | 17.9 | 3805 ms | 4516 | **530.0x** | **252.7x** | 6 | 0 | 0 |
| infer | 18 | 132.1 ms | 164.7 | 2759 ms | 4404 | **20.9x** | **26.7x** | 6 | 0 | 0 |
| sig | 16 | 848.3 ms | 1028.9 | 2265 ms | 2381 | **2.7x** | **2.3x** | 8 | 0 | 0 |
| sigx | 9 | 1018.7 ms | 1090.8 | 2384 ms | 2708 | **2.3x** | **2.5x** | 3 | 0 | 0 |
| oblig | 12 | 4.5 ms | 9.9 | 2377 ms | 2662 | **534.1x** | **269.7x** | 6 | 0 | 0 |
| derive | 9 | 1171.4 ms | 1299.5 | 3299 ms | 4842 | **2.8x** | **3.7x** | 3 | 0 | 0 |
| generic | 14 | 4.1 ms | 11.0 | 2282 ms | 2452 | **551.3x** | **222.7x** | 4 | 0 | 0 |
| rpit | 18 | 950.1 ms | 1303.5 | 2265 ms | 2659 | **2.4x** | **2.0x** | 6 | 0 | 0 |

### `goport-util`: 37,978 lines, 81 files

ts-rust `b11b618ae`, lust `c938c0f9b`, rustc 1.99.0 (b940084d7 2026-09-28), 16 vCPU, 6 probes per kind, 146 states. Daemon first check 399 ms, no-op 1.24 ms.

| Kind | ok states | lust median | lust p95 | cargo median | cargo p95 | Ratio median | Ratio p95 | reject states | false accepts | warm != cold |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| body | 12 | 4.1 ms | 10.3 | 394 ms | 436 | **96.8x** | **42.3x** | 6 | 0 | 0 |
| infer | 18 | 41.2 ms | 48.8 | 374 ms | 423 | **9.1x** | **8.7x** | 6 | 0 | 0 |
| sig | 16 | 113.5 ms | 147.6 | 331 ms | 370 | **2.9x** | **2.5x** | 8 | 0 | 0 |
| sigx | 9 | 132.2 ms | 149.6 | 382 ms | 414 | **2.9x** | **2.8x** | 3 | 0 | 0 |
| oblig | 12 | 3.1 ms | 7.3 | 344 ms | 362 | **109.8x** | **49.8x** | 6 | 0 | 0 |
| derive | 8 | 151.0 ms | 166.6 | 481 ms | 711 | **3.2x** | **4.3x** | 4 | 0 | 0 |
| generic | 14 | 2.9 ms | 5.7 | 326 ms | 375 | **113.9x** | **66.5x** | 4 | 0 | 0 |
| rpit | 15 | 103.2 ms | 121.9 | 280 ms | 326 | **2.7x** | **2.7x** | 5 | 0 | 0 |
<!-- tsedit-table:end -->

## Larger slices

`tsrust-all` is all of ts_goport (671,746 lines with both part crates and
the smallvec and xxhash_rust stand-ins) pruned to what lust takes today:
160k lines. A 200k-line slice needs lust to take more of ts-rust; the top
blocker is `Default` for `Box<[T]>` (derive(Default) over a `Box<[T]>`
field in core.rs `FileNodeBind` and `Table`, checker `FlatMap`: 131k lines
of cascade). docs/real-coverage.md ranks the blockers per root cause.
