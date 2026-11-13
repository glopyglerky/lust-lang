# Hash core tests

Gate for `src/runtime/hash.c` (erased hashbrown core, FxHasher, and
the IndexMap core) against Rust 1.99. Run:

```sh
export LUST_TCC_HOME=/path/to/third_party/tcc   # or scripts/build-tcc.sh
tests/hash/test.sh            # 40k ops/stream, 2 seeds, ~15 s
tests/hash/test.sh --full     # 160k ops/stream, 3.2M checked ops
tests/hash/test.sh --bench    # adds interleaved ns/op rounds
```

## How it works

`gen_ops.py` writes a deterministic op stream (splitmix64, seeded).
The Rust oracle in `oracle/` (rustc-hash 2.1.2, indexmap 2.14.0,
hashbrown 0.17.1, the version in Rust 1.99 std and in ts-rust's
Cargo.lock) and the C harness in `harness.c` (pinned tcc; also built
with gcc once per run) both execute the stream and print the op
result plus `len capacity [entries in iteration order]` after every
op. The gate is `cmp` on the two outputs: zero differing bytes.

Streams mix insert (both insert and entry paths), get, remove,
reserve, shrink_to, shrink_to_fit, clear, retain, extract_if, drain
with early drop, clone, and new/with_capacity resets. Remove storms
every 4000 ops empty most of the key space so tombstones build up and
the refill forces hashbrown's rehash-in-place and resize paths. Index
streams add shift/swap remove by key and by index, pop, shift_insert,
reserve_exact, drain ranges, truncate, reverse, move_index, and
swap_indices.

Modes: `fx64` FxHashMap<u64,u64>, `fx32` FxHashMap<u32,u64>, `fxstr`
FxHashMap<String,u64> (keys are made from ids; state prints the id),
`sip64` std HashMap over the deprecated `SipHasher` (SipHash-2-4,
keys 0,0; the C side is a streaming port of core's sip.rs, so the
table is proven hasher-agnostic against real std hashing), `set64`
FxHashSet<u64>, `im64`/`imstr` IndexMap with FxBuildHasher, and
`hb8` over `hashbrown::HashTable` directly. `hb8` runs twice: once
with the normal width-16 builds, and once with an oracle built with
`-C target-feature=-sse,-sse2` (hashbrown falls back to its generic
8-byte group) against the C harness built with
`-DLUST_HASH_GROUP_WIDTH=8`. Width-8 output differs from width-16
output on the same stream, and each build matches its own oracle:
that is the aarch64 NEON width pin, proven as far as this host can
(the NEON group is also 8 bytes; its bitmask packing differs, the
probe order does not).

`fxvals` compares `FxBuildHasher::hash_one` against `lust_fx_*`
primitives for integer, bool, char, str, String, byte slice, and
tuple inputs plus 2000 random values. `fxvectors` asserts the
published rustc-hash 2.1.2 test values in C directly. `edges`
asserts small-table capacities per group width, a ZST table, the
empty-clear tombstone quirk, and an IndexMap capacity corner.

Current result (2026-10-12): `test.sh --full` passes every mode,
3,200,000 checked operations, zero differing bytes, and the harness
is ASan/UBSan clean on sampled streams.

## Bench (same host, medians of interleaved rounds)

200k inserts into a 300k key space, 200k hit+miss lookups, 100k
removes, 20 full iterations. Rust is the release oracle with debug
assertions on; C is the pinned tcc build. Numbers move with host
load; ratios are the signal.

| op | Rust std FxHashMap | C core (tcc) | C core (gcc -O2) |
|---|---|---|---|
| insert | ~67 ns | ~460 ns | ~190 ns |
| get (hit+miss avg) | ~13 ns | ~190 ns | ~108 ns |
| remove | ~35 ns | ~420 ns | ~240 ns |
| iterate (per item) | ~2 ns | ~30 ns | ~5 ns |

IndexMap insert/get: Rust ~185/~29 ns, C (tcc) ~1040/~340 ns. The
gap is tcc codegen plus callback and rehash calls, not extra table
work: gcc closes most of it. If dev-build speed needs more, the wins
are in the typed layer inlining hash and eq instead of callbacks, not
in changing table behavior.

## What the typed lust layer (W-STD-B, W-DEPS) still needs

Nothing from this C API is wired into generated code yet. To use it:

1. Amalgamate like float: splice `hash.h`/`hash.c` in
   `cgen::runtime_source` next to `float.h`, and add one
   `#include "hash.h"` line at the end of `core.c`. Until then this
   file is only built by these tests.
2. Element layout is the caller's: pass the Rust pair size and align
   for `(K, V)` (and `(T, ())` for sets). Only sizes 0 to 3 change
   bucket counts, but pass the real size everywhere so `capacity()`
   stays right.
3. Call the entry flavor your API surface uses (std, hashbrown
   crate, and HashTable differ; all three are mapped in `hash.h`).
   This is the easiest place to silently diverge; the `e` op in
   these streams exists because a first version got it wrong.
4. Hash byte streams are above FxHasher: the lust `Hash` impls must
   write what core writes (str 0xff suffix, slice length prefix,
   enum discriminant as isize, and so on). `fxvals` pins the common
   ones. SipHash-1-3 for std maps is source per ARCH 3.2; only the
   hasher state type is needed there, the table takes any u64 hash.
5. Drops: pass drop callbacks to clear/retain/drain_finish/drop.
   A panicking hasher, eq, clone, or drop has no C counterpart here:
   hashbrown restores a valid table on panic (scope guards in
   rehash-in-place, clear, resize). The typed layer must not let a
   lust panic unwind through these calls yet; that is a known gap,
   shared with the rest of the runtime until unwind support lands.
6. `try_reserve` and fallible allocation are not exposed: allocation
   failure and capacity overflow print the Rust text and abort, like
   `lust_oom`. Add a fallible variant when std needs one.
7. IndexMap surface not in the core: entry objects, slicing, sort
   (sort the entries through `lust_index_entry_mut`, then call
   `lust_index_rebuild`), serde, rayon. `get_disjoint_mut` is plain
   finds. The entries `Bucket` byte size here places fields by
   alignment like Rust, but the real size must come from the lust
   layout engine later; entry counts (what `capacity()` returns) do
   not depend on it.
