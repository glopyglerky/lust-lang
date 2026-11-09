/* Erased hash table core for the lust runtime: byte-exact with the
   hashbrown 0.17.1 table inside Rust 1.99 std (the table under std
   HashMap/HashSet, rustc-hash FxHashMap/FxHashSet, hashbrown HashTable,
   and the index table of indexmap 2.14.0, which pins hashbrown 0.17.1
   in ts-rust's Cargo.lock).

   Exactness scope: probe sequence, control bytes, growth, reserve,
   shrink, erase (tombstone vs empty), rehash in place, and iteration
   order all match hashbrown 0.17.1 for the same hash values, element
   size, and operation sequence. Iteration order is increasing bucket
   index. Capacity (items + growth_left) matches, so HashMap::capacity
   matches. Element memory layout inside the allocation does NOT match
   hashbrown (buckets are stored forward, not reversed); that layout is
   not observable through the std or indexmap APIs.

   Group width is part of the probe sequence (the probe stride grows by
   Group::WIDTH), so it is a compile-time target pin, not a free choice:
     LUST_HASH_GROUP_WIDTH 16  x86_64 with SSE2 (the default here)
     LUST_HASH_GROUP_WIDTH 8   aarch64 with NEON (uint8x8_t group) and
                               the generic 64-bit word group
   One binary matches one target. A width-16 table and a width-8 table
   give different iteration orders for the same inserts, exactly as
   hashbrown does across those targets. The group scan itself is SWAR
   over 8-byte chunks in this core; only the width (an observable) is
   pinned, not the instruction choice (not observable).

   The caller hashes. Every lookup or insert takes the full 64-bit hash
   of the probe key as a value. Operations that may rehash (reserve,
   shrink, insert fallbacks) also take a hash_of callback that returns
   the hash of a stored element, like hashbrown's `hasher` argument.
   Equality is a callback over a stored element with a caller context
   that holds the probe key. So FxHasher, SipHash, or any other BuildHasher
   works with no change here. FxHasher (rustc-hash 2.1.2) is provided
   below for convenience; a typed layer may also implement it in source.

   Rust operation -> this API (std HashMap layer):
     HashMap::new                 lust_hash_init (empty singleton)
     HashMap::with_capacity(n)    lust_hash_init_with_capacity
     map.get / get_mut / contains lust_hash_find
     map.insert                   lust_hash_find_or_insert, then replace
                                  the value in the slot, or
                                  lust_hash_insert_at + write the slot
     VacantEntry::insert          three flavors differ, keep them apart:
                                  std HashMap: entry() itself reserves 1
                                  when vacant (hashbrown rustc_entry),
                                  then lust_hash_insert_no_grow_slot;
                                  hashbrown crate HashMap: entry() only
                                  finds, insert re-probes through
                                  lust_hash_insert_unique_slot;
                                  hashbrown HashTable: entry reserves in
                                  the search, insert is lust_hash_insert_at
                                  at the found index (indexmap side table)
     map.remove / remove_entry    lust_hash_remove
     map.reserve(n)               lust_hash_reserve
     map.shrink_to_fit / shrink_to lust_hash_shrink_to (0 for to_fit)
     map.clear                    lust_hash_clear
     map.retain                   lust_hash_retain
     map.drain                    lust_hash_drain_* (yields in iteration
                                  order; finish drops the rest, then the
                                  table is empty with capacity kept)
     map.clone                    lust_hash_clone_into (same bucket count,
                                  same indices, elements cloned in order)
     iter / keys / values         lust_hash_iter_*
   HashSet uses the same core with element = the set element. std HashSet
   stores (T, ()) pairs in Rust, which has the same size and align as T
   when the unit adds no padding; the typed layer owns that choice. If it
   wants exact capacity for sets it must pass the pair size, because
   capacity_to_buckets depends on element size for tiny tables.

   Slot pointers are invalidated by any later mutating call on the same
   table, like Rust references into a map. Callbacks must not mutate the
   table. On allocation failure or capacity overflow the core prints the
   Rust message and aborts, like the rest of this runtime; there is no
   try_reserve error path yet (typed layer gap, see tests/hash/README).

   Wiring: core.c ends with `#include "hash.h"`, and
   cgen::runtime_source splices this header (and hash.c in the runtime
   unit) into the amalgamation the same way as float.h. Later std work
   calls it from source wrappers. Tests include this file directly.
   Correctness is gated by tests/hash/ against rustc 1.99 oracles. */
#ifndef LUST_RT_HASH_H
#define LUST_RT_HASH_H

#include <stddef.h>
#include <stdint.h>

#ifndef LUST_HASH_GROUP_WIDTH
#if defined(__x86_64__) || defined(__i386__)
#define LUST_HASH_GROUP_WIDTH 16
#else
#define LUST_HASH_GROUP_WIDTH 8
#endif
#endif

/* Control byte values, as in hashbrown's Tag. Full tags are 0..127. */
#define LUST_HASH_EMPTY 0xFFu
#define LUST_HASH_DELETED 0x80u

/* The full tag for a hash: its top 7 bits (64-bit hashes). */
static inline uint8_t lust_hash_tag(uint64_t hash) { return (uint8_t)((hash >> 57) & 0x7Fu); }

/* Hash of a stored element. `elem` points at the element bytes. */
typedef uint64_t (*lust_hash_of_fn)(void *ctx, const void *elem);
/* True when the stored element `elem` matches the probe key in `ctx`. */
typedef int (*lust_hash_eq_fn)(void *ctx, const void *elem);
/* Drops one stored element in place. May be NULL when elements are plain. */
typedef void (*lust_hash_drop_fn)(void *ctx, void *elem);
/* Retain predicate: true keeps the element. May mutate the element. */
typedef int (*lust_hash_keep_fn)(void *ctx, void *elem);
/* Clone `src` element into the uninitialized `dst` slot. */
typedef void (*lust_hash_clone_fn)(void *ctx, void *dst, const void *src);

/* One erased hash table. Zero the struct, then call an init function.
   `ctrl` holds buckets + GROUP_WIDTH control bytes (the tail mirrors the
   head, as in hashbrown). An unallocated table is the empty singleton:
   bucket_mask 0 with `ctrl` pointing at static empty tags. */
typedef struct {
    /* The four fields a lookup reads come first, in 32 bytes: a map inside a struct then
       touches one cache line for its header more often (hashbrown's header is 32 bytes). */
    uint8_t *ctrl;
    uint64_t bucket_mask;
    uint8_t *data;       /* bucket i at data + i * elem_size; NULL if singleton or ZST */
    uint64_t elem_size;
    uint64_t items;
    uint64_t growth_left;
    void *alloc_base;    /* raw malloc pointer, NULL if singleton */
    uint64_t elem_align;
} lust_hash_table;

/* The empty singleton's control bytes (Group::static_empty). A unit's header gets these
   declarations after inl.h, whose macros carry the names of the five functions that are in
   parentheses below: the parentheses keep those macros out of the declarations. */
extern const uint8_t lust_hash_static_empty[16];

/* Creates the empty table (no allocation), for elements of this size and
   align. Rust layout rules apply: size is a multiple of align, align is a
   power of two. Size 0 (ZST elements) is supported. */
void (lust_hash_init)(lust_hash_table *t, uint64_t elem_size, uint64_t elem_align);
/* Creates a table that holds `capacity` elements without growing.
   Capacity 0 gives the empty table. Bucket count follows hashbrown
   capacity_to_buckets, including its small-table minimums. */
void lust_hash_init_with_capacity(lust_hash_table *t, uint64_t elem_size, uint64_t elem_align, uint64_t capacity);
/* Frees the allocation. With a drop function, drops each element first in
   iteration order (Rust drop order for maps in practice: bucket order).
   Without one (drop == NULL), elements are not dropped. Either way the
   table becomes the empty singleton and may be reused. */
void (lust_hash_drop_table)(lust_hash_table *t, lust_hash_drop_fn drop, void *ctx);

uint64_t (lust_hash_len)(const lust_hash_table *t);
uint64_t (lust_hash_capacity)(const lust_hash_table *t); /* items + growth_left */
uint64_t lust_hash_buckets(const lust_hash_table *t);  /* 1 for the singleton */
int (lust_hash_is_empty)(const lust_hash_table *t);

/* The slot for bucket `index`, or NULL when it is not full. */
void *lust_hash_bucket(const lust_hash_table *t, uint64_t index);
/* The control byte for bucket `index` (tests and debug). */
uint8_t lust_hash_ctrl(const lust_hash_table *t, uint64_t index);

/* Finds the element with this hash that `eq` accepts. Returns its slot, or
   NULL. `out_index` (may be NULL) receives the bucket index on a hit. */
void *lust_hash_find(const lust_hash_table *t, uint64_t hash, lust_hash_eq_fn eq, void *eq_ctx, uint64_t *out_index);

/* HashMap::insert and entry search: reserves room for one more element
   first (exactly like hashbrown, even when the key turns out to exist),
   then searches. Returns 1 and the slot of the existing element, or 0 and
   in `out_index` the bucket index to insert at with lust_hash_insert_at. */
int lust_hash_find_or_insert(lust_hash_table *t, uint64_t hash, lust_hash_eq_fn eq, void *eq_ctx,
                             lust_hash_of_fn hash_of, void *hash_ctx, void **out_slot, uint64_t *out_index);
/* Records a new element at a bucket index from lust_hash_find_or_insert
   (or from a hashbrown HashTable vacant entry) and returns the slot to
   write the element bytes into. The slot is recorded before the write,
   as in hashbrown; write it before any other call on this table. */
void *lust_hash_insert_at(lust_hash_table *t, uint64_t hash, uint64_t index);
/* RawTable::insert for a key the caller knows is absent: probes for a
   free bucket, grows only if that bucket is EMPTY and no growth is left
   (a DELETED bucket is reused without growing), records the element,
   and returns the slot to write into. `out_index` may be NULL. */
void *lust_hash_insert_unique_slot(lust_hash_table *t, uint64_t hash, lust_hash_of_fn hash_of, void *hash_ctx,
                                   uint64_t *out_index);
/* RawTable::insert_no_grow for a key the caller knows is absent, after
   the caller reserved room (the std entry path: std entry() finds, then
   reserves 1 while still vacant, and only this insert runs after). The
   table must have a free bucket after that reserve. Returns the slot to
   write into. `out_index` may be NULL. */
void *lust_hash_insert_no_grow_slot(lust_hash_table *t, uint64_t hash, uint64_t *out_index);

/* Removes the matching element. Returns 1 on a hit; when `out_elem` is
   not NULL the element bytes are copied there first (ownership moves to
   the caller). The bucket becomes EMPTY or DELETED by hashbrown's erase
   rule (a full group of neighbours with no empty bucket in it). */
int lust_hash_remove(lust_hash_table *t, uint64_t hash, lust_hash_eq_fn eq, void *eq_ctx, void *out_elem);
/* Erases bucket `index`, which must be full. `out_elem` as above; pass
   NULL and drop the element yourself before or after as needed. */
void lust_hash_erase_at(lust_hash_table *t, uint64_t index, void *out_elem);
/* The bucket index of a full bucket's slot (an OccupiedEntry's). With a
   zero-sized element every slot has one address and the map holds at most
   one element: its bucket. */
uint64_t lust_hash_index_of(const lust_hash_table *t, const void *slot);

/* Ensures room for `additional` more elements. May rehash in place (same
   bucket count, tombstones cleared) when the new length fits in half the
   current capacity, else resizes, exactly as hashbrown reserve does. */
void lust_hash_reserve(lust_hash_table *t, uint64_t additional, lust_hash_of_fn hash_of, void *hash_ctx);
/* Shrinks toward `min_capacity` (0 is shrink_to_fit; the result still
   holds all elements). An emptied table is freed back to the singleton
   only when the clamped minimum is 0, as in hashbrown shrink_to. */
void lust_hash_shrink_to(lust_hash_table *t, uint64_t min_capacity, lust_hash_of_fn hash_of, void *hash_ctx);
/* Removes all elements (dropped in iteration order when drop != NULL).
   The allocation and capacity are kept. */
void lust_hash_clear(lust_hash_table *t, lust_hash_drop_fn drop, void *ctx);
/* Keeps the elements `keep` accepts, drops the rest, in iteration order.
   The predicate runs once per element in bucket order, as in Rust. */
void lust_hash_retain(lust_hash_table *t, lust_hash_keep_fn keep, void *keep_ctx, lust_hash_drop_fn drop,
                      void *drop_ctx);
/* Clones into `dst`, which must be an empty singleton for the same element
   layout (call lust_hash_init first). The clone gets the same bucket
   count and bucket indices; elements are cloned in iteration order. */
void lust_hash_clone_into(lust_hash_table *dst, const lust_hash_table *src, lust_hash_clone_fn clone_elem, void *ctx);

/* Iteration in bucket order. `left` counts down from len. Do not mutate
   the table while iterating, except through lust_hash_retain/drain. */
typedef struct {
    const lust_hash_table *t;
    uint64_t next_index;   /* next group base to load */
    uint64_t group_base;   /* base index of the loaded group */
    uint32_t group_bits;   /* full buckets left in the loaded group */
    uint64_t left;
} lust_hash_iter;
void lust_hash_iter_init(lust_hash_iter *it, const lust_hash_table *t);
/* Returns the next slot, or NULL at the end. `out_index` may be NULL. */
void *lust_hash_iter_next(lust_hash_iter *it, uint64_t *out_index);

/* Drain: yields slots in iteration order. The caller moves each element
   out of its slot before the next call. lust_hash_drain_finish drops the
   elements not yet yielded (when drop != NULL) and leaves the table
   empty with its capacity kept, matching a dropped Rust Drain. */
typedef struct {
    lust_hash_table *t;
    uint64_t next_index;
    uint64_t group_base;
    uint32_t group_bits;
    uint64_t left;
} lust_hash_drain;
void lust_hash_drain_init(lust_hash_drain *d, lust_hash_table *t);
void *lust_hash_drain_next(lust_hash_drain *d, uint64_t *out_index);
void lust_hash_drain_finish(lust_hash_drain *d, lust_hash_drop_fn drop, void *ctx);

/* ---------------- FxHasher (rustc-hash 2.1.2, 64-bit) ----------------
   State is one usize. Each write_* is the Hasher method of the same name
   as rustc-hash implements it on 64-bit targets (non-nightly build, the
   one ts-rust uses): write_u8/16/32/64/usize add once, write_u128 adds
   the low then the high 64 bits, write(bytes) goes through hash_bytes,
   and finish rotates left by 26. Signed and length/string helpers use
   the Rust core defaults: write_iN is write_uN of the same bits,
   write_length_prefix is write_usize, write_str is write(bytes) then
   write_u8(0xff). Hash byte streams above this layer (which write_*
   calls a type makes) belong to the typed layer and must match core:
   str writes the 0xff suffix, slices write a length prefix, a derived
   enum writes its discriminant as isize, and so on.
   On 32-bit targets rustc-hash differs (K, rotation 15, extra adds for
   u64/u128, the 32-bit multiply_mix); this core is 64-bit only, like
   both tier 1 targets. */
typedef struct {
    uint64_t hash;
} lust_fx_hasher;

lust_fx_hasher lust_fx_new(void);               /* seed 0 */
lust_fx_hasher lust_fx_with_seed(uint64_t seed);
void lust_fx_write(lust_fx_hasher *h, const void *bytes, uint64_t len);
void lust_fx_write_u8(lust_fx_hasher *h, uint8_t v);
void lust_fx_write_u16(lust_fx_hasher *h, uint16_t v);
void lust_fx_write_u32(lust_fx_hasher *h, uint32_t v);
void lust_fx_write_u64(lust_fx_hasher *h, uint64_t v);
void lust_fx_write_u128(lust_fx_hasher *h, uint64_t lo, uint64_t hi);
void lust_fx_write_usize(lust_fx_hasher *h, uint64_t v);
void lust_fx_write_i8(lust_fx_hasher *h, int8_t v);
void lust_fx_write_i16(lust_fx_hasher *h, int16_t v);
void lust_fx_write_i32(lust_fx_hasher *h, int32_t v);
void lust_fx_write_i64(lust_fx_hasher *h, int64_t v);
void lust_fx_write_i128(lust_fx_hasher *h, uint64_t lo, uint64_t hi);
void lust_fx_write_isize(lust_fx_hasher *h, int64_t v);
void lust_fx_write_length_prefix(lust_fx_hasher *h, uint64_t len);
void lust_fx_write_str(lust_fx_hasher *h, const void *bytes, uint64_t len);
uint64_t lust_fx_finish(const lust_fx_hasher *h);
/* The wyhash-inspired byte hash inside write(): exposed for tests and
   for callers that hash one byte string with no other writes (finish
   still applies: hash_one(bytes) is not lust_fx_hash_bytes alone). */
uint64_t lust_fx_hash_bytes(const void *bytes, uint64_t len);
/* One-shot helpers: a fresh hasher, one write, finish. These match
   FxBuildHasher::hash_one for the Rust types of the same name. */
uint64_t lust_fx_hash_u32(uint32_t v);
uint64_t lust_fx_hash_u64(uint64_t v);
uint64_t lust_fx_hash_usize(uint64_t v);
uint64_t lust_fx_hash_str(const void *bytes, uint64_t len);

/* ---------------- SipHash-1-3 (std DefaultHasher) ----------------
   core::hash::sip::Hasher<Sip13Rounds> as Rust 1.99 has it, behind
   std::hash::DefaultHasher: a byte-stream hash, so every write_* is a
   write of the value's little-endian bytes, and write_str adds 0xff.
   DefaultHasher::new() is keys (0, 0); RandomState draws keys (lust:
   a fixed start, k0 bumped per RandomState, as std bumps its per-thread
   keys; map order under RandomState is not an observable). The field
   order and size match Rust's (72 bytes). */
typedef struct {
    uint64_t k0;
    uint64_t k1;
    uint64_t length;
    uint64_t v0;
    uint64_t v2;
    uint64_t v1;
    uint64_t v3;
    uint64_t tail;
    uint64_t ntail;
} lust_sip13;
typedef struct {
    uint64_t k0;
    uint64_t k1;
} lust_random_state;

lust_sip13 lust_sip13_new_with_keys(uint64_t k0, uint64_t k1);
void lust_sip13_write(lust_sip13 *h, const void *bytes, uint64_t len);
uint64_t lust_sip13_finish(const lust_sip13 *h);
/* RandomState::new(): the next keys (std: per-thread, k0 + 1 each). */
lust_random_state lust_random_state_new(void);

/* ---------------- IndexMap core (indexmap 2.14.0) ----------------
   Entries are kept in insertion order in one erased Vec; a side
   lust_hash_table maps key hash to entry index (hashbrown HashTable of
   usize there, this core here), so the probing, growth, and tombstone
   behavior of the side table also matches. Each entry slot holds, in
   order: u64 hash, the key bytes, then the value bytes, each aligned as
   the caller says. The caller supplies sizes and aligns for key and
   value at init and hashes keys itself (IndexMap stores the u64 hash it
   computed from the map hasher; capacity decisions that depend on the
   entry struct size use the same total entry size here).

   Semantics matched: insertion order iteration, insert_full index,
   get by key and by index, swap_remove vs shift_remove (and the _index
   and pop forms), shift_insert at a position, reserve and reserve_exact
   (including indexmap's jump of the entries Vec capacity toward the
   side table capacity), shrink_to and shrink_to_fit, clear, retain
   (entries compacted in order, then the side table is cleared and
   rebuilt in entry order), reverse, swap_indices, move_index, truncate
   and range drain through the same erase_indices strategy choices, and
   capacity() = min(side table capacity, entries capacity).
   Sorting the entries is the typed layer's job (Rust std sort); after
   it reorders entry bytes with lust_index_entries_mut it must call
   lust_index_rebuild, like indexmap's with_entries.
   IndexSet is the same core with value size 0. */
typedef struct {
    lust_hash_table indices; /* element: uint64_t entry index */
    uint8_t *entries;        /* entries_cap * entry_size bytes */
    void *entries_alloc;     /* raw malloc pointer */
    uint64_t len;
    uint64_t entries_cap;
    uint64_t entry_size;     /* hash + key + value, padded to entry_align */
    uint64_t entry_align;
    uint64_t hash_offset;
    uint64_t key_offset;
    uint64_t key_size;
    uint64_t value_offset;
    uint64_t value_size;
} lust_index_table;

/* Drop one entry given its key and value slot pointers. */
typedef void (*lust_index_drop_fn)(void *ctx, void *key, void *value);
/* True when the stored key `key` matches the probe key in `ctx`. */
typedef int (*lust_index_eq_fn)(void *ctx, const void *key);
/* Retain predicate over one entry's key and value. True keeps it. */
typedef int (*lust_index_keep_fn)(void *ctx, void *key, void *value);

void lust_index_init(lust_index_table *t, uint64_t key_size, uint64_t key_align, uint64_t value_size,
                     uint64_t value_align);
void lust_index_init_with_capacity(lust_index_table *t, uint64_t key_size, uint64_t key_align, uint64_t value_size,
                                   uint64_t value_align, uint64_t capacity);
/* Drops all entries in order (when drop != NULL), frees both tables,
   and leaves an empty table with the same key/value layout. */
void lust_index_drop_table(lust_index_table *t, lust_index_drop_fn drop, void *ctx);

uint64_t lust_index_len(const lust_index_table *t);
uint64_t lust_index_capacity(const lust_index_table *t);
int lust_index_is_empty(const lust_index_table *t);

/* Key/value slots of the entry at position `index`, or NULL if out of
   range. Either out pointer may be NULL. Entry order is iteration order. */
int lust_index_get_index(const lust_index_table *t, uint64_t index, void **out_key, void **out_value);
/* Raw entry bytes for position `index` (hash word first). Use with
   lust_index_rebuild after an in-place reorder such as sort. */
void *lust_index_entry_mut(lust_index_table *t, uint64_t index);
uint64_t lust_index_entry_hash(const lust_index_table *t, uint64_t index);
/* The position of the matching key, or UINT64_MAX. */
uint64_t lust_index_get_index_of(const lust_index_table *t, uint64_t hash, lust_index_eq_fn eq, void *eq_ctx);
int lust_index_get(const lust_index_table *t, uint64_t hash, lust_index_eq_fn eq, void *eq_ctx, void **out_key,
                   void **out_value, uint64_t *out_index);

/* IndexMap::insert_full: if the key exists, returns its position, sets
   *out_replaced to 1, and gives the value slot in *out_value_slot (the
   caller swaps the new value in; the key is kept, as in Rust). Else
   appends a new entry and returns its position with *out_replaced 0 and
   the key/value slots to write into. The key bytes for the probe are
   read through `eq`; on insert the caller writes the real key bytes. */
uint64_t lust_index_insert_full(lust_index_table *t, uint64_t hash, lust_index_eq_fn eq, void *eq_ctx, int *out_replaced,
                                void **out_key_slot, void **out_value_slot);
/* Appends a key the caller knows is absent (entry API vacant paths,
   extend with unique keys). Returns the position; slots as above. */
uint64_t lust_index_insert_unique(lust_index_table *t, uint64_t hash, void **out_key_slot, void **out_value_slot);
/* VacantEntry::shift_insert: inserts a known-absent key at `index`
   (0..=len), shifting later entries one position up. */
void lust_index_shift_insert_unique(lust_index_table *t, uint64_t index, uint64_t hash, void **out_key_slot,
                                    void **out_value_slot);

/* Removals. The removed key and value bytes are copied to out_key /
   out_value when they are not NULL (ownership moves out). The by-key
   forms also report the position the entry had. */
int lust_index_swap_remove(lust_index_table *t, uint64_t hash, lust_index_eq_fn eq, void *eq_ctx, void *out_key,
                           void *out_value, uint64_t *out_index);
int lust_index_shift_remove(lust_index_table *t, uint64_t hash, lust_index_eq_fn eq, void *eq_ctx, void *out_key,
                            void *out_value, uint64_t *out_index);
int lust_index_swap_remove_index(lust_index_table *t, uint64_t index, void *out_key, void *out_value);
int lust_index_shift_remove_index(lust_index_table *t, uint64_t index, void *out_key, void *out_value);
int lust_index_pop(lust_index_table *t, void *out_key, void *out_value);

void lust_index_reserve(lust_index_table *t, uint64_t additional);
void lust_index_reserve_exact(lust_index_table *t, uint64_t additional);
void lust_index_shrink_to(lust_index_table *t, uint64_t min_capacity); /* 0 is shrink_to_fit */
void lust_index_clear(lust_index_table *t, lust_index_drop_fn drop, void *ctx);
void lust_index_truncate(lust_index_table *t, uint64_t len, lust_index_drop_fn drop, void *ctx);
/* Keeps accepted entries in order, drops the rest in order, then rebuilds
   the side table when anything was removed (indexmap retain_in_order).
   The predicate runs in entry order and may mutate key/value except it
   must not change any key's hash or equality, as in Rust. */
void lust_index_retain(lust_index_table *t, lust_index_keep_fn keep, void *keep_ctx, lust_index_drop_fn drop,
                       void *drop_ctx);
void lust_index_reverse(lust_index_table *t);
void lust_index_swap_indices(lust_index_table *t, uint64_t a, uint64_t b);
void lust_index_move_index(lust_index_table *t, uint64_t from, uint64_t to);
/* Removes the entry range [start, end), dropping those entries in order
   (when drop != NULL). Matches the erase_indices strategies of indexmap
   drain/truncate/splice for side table state; a lazy Rust Drain iterator
   that can be dropped midway belongs to the typed layer, which can
   drive the same end state from get_index + this call. */
void lust_index_drain_range(lust_index_table *t, uint64_t start, uint64_t end, lust_index_drop_fn drop, void *ctx);
/* Clears and rebuilds the side table from entry order and stored hashes.
   Required after the caller reorders or rewrites entry bytes. */
void lust_index_rebuild(lust_index_table *t);

#if !defined(__TINYC__) && !defined(LUST_IMPL) && !defined(LUST_NO_INL)
/* Units an optimizing C compiler builds (inl.h's rule: not tcc, the runtime unit or
   -DLUST_NO_INL) run a hash lookup's probe in the caller's unit, as hashbrown's
   monomorphized find runs in rustc's code: the key compare is the caller's eq glue, a known
   fn that clang calls directly or inlines. hash.c's order and results (lh_find_inner):
   h1 = hash & mask, a triangular stride of one group, matches in bit order, stop at a group
   with an EMPTY byte. The group test is hash.c's: one SSE2 load on x86_64 (vector builtins:
   units read no system header), its SWAR otherwise (a SWAR tag test may also flag the byte
   above a match, a candidate eq rejects). It returns the tag matches and sets the EMPTY and
   the EMPTY-or-DELETED masks (a caller that does not read a mask costs nothing for it). */
#if LUST_HASH_GROUP_WIDTH == 16 && defined(__SSE2__)
typedef char lust_hash_v16_ __attribute__((vector_size(16)));
static inline uint32_t lust_hash_group_(const uint8_t *ctrl, uint8_t tag, uint32_t *empty, uint32_t *eod) {
    lust_hash_v16_ g, zero = {0};
    __builtin_memcpy(&g, ctrl, 16);
    *empty = (uint32_t)__builtin_ia32_pmovmskb128((lust_hash_v16_)(g == zero + (char)LUST_HASH_EMPTY));
    *eod = (uint32_t)__builtin_ia32_pmovmskb128(g);
    return (uint32_t)__builtin_ia32_pmovmskb128((lust_hash_v16_)(g == zero + (char)tag));
}
#else
static inline uint32_t lust_hash_spread_bits_(uint64_t spread) {
    return (uint32_t)((((spread >> 7) & 0x0101010101010101ull) * 0x0102040810204080ull) >> 56);
}
static inline uint32_t lust_hash_bytes_(uint64_t w, uint8_t b) {
    uint64_t x = w ^ (b * 0x0101010101010101ull);
    return lust_hash_spread_bits_((x - 0x0101010101010101ull) & ~x & 0x8080808080808080ull);
}
static inline uint32_t lust_hash_group_(const uint8_t *ctrl, uint8_t tag, uint32_t *empty, uint32_t *eod) {
    uint32_t tb = 0, eb = 0, db = 0;
    for (uint32_t off = 0; off < LUST_HASH_GROUP_WIDTH; off += 8) {
        uint64_t w;
        __builtin_memcpy(&w, ctrl + off, 8);
        tb |= lust_hash_bytes_(w, tag) << off;
        eb |= lust_hash_bytes_(w, LUST_HASH_EMPTY) << off;
        db |= lust_hash_spread_bits_(w & 0x8080808080808080ull) << off;
    }
    *empty = eb;
    *eod = db;
    return tb;
}
#endif
static inline void *lust_hash_find_inl_(const lust_hash_table *t, uint64_t hash, lust_hash_eq_fn eq, void *ctx, uint64_t *out) {
    uint8_t tag = lust_hash_tag(hash);
    uint64_t mask = t->bucket_mask, pos = hash & mask, stride = 0;
    for (;;) {
        uint32_t empty, eod, m = lust_hash_group_(t->ctrl + pos, tag, &empty, &eod);
        while (m) {
            uint64_t index = (pos + (uint64_t)__builtin_ctz(m)) & mask;
            m &= m - 1;
            void *slot = (void *)(t->data + index * t->elem_size);
            if (eq(ctx, slot)) {
                if (out) *out = index;
                return slot;
            }
        }
        if (empty) return 0;
        stride += LUST_HASH_GROUP_WIDTH;
        pos = (pos + stride) & mask;
    }
}
/* The function reserves room for one first, which may move every element; with room that is
   a no-op, so the whole search runs here: lh_find_or_insert_inner's single pass, which notes
   the first EMPTY or DELETED bucket on its way and stops at a group with an EMPTY byte. A
   miss returns that insert index (fix_insert_index: in a table smaller than a group, a
   mirror byte can name a full bucket; then it is group 0's first EMPTY or DELETED). A full
   table calls the function (it grows first). */
static inline int lust_hash_find_or_insert_inl_(lust_hash_table *t, uint64_t hash, lust_hash_eq_fn eq, void *ctx,
                                                lust_hash_of_fn hash_of, void *hash_ctx, void **out_slot, uint64_t *out_index) {
    if (__builtin_expect(t->growth_left == 0, 0))
        return lust_hash_find_or_insert(t, hash, eq, ctx, hash_of, hash_ctx, out_slot, out_index);
    uint8_t tag = lust_hash_tag(hash);
    uint64_t mask = t->bucket_mask, pos = hash & mask, stride = 0, insert = 0;
    int have_insert = 0;
    for (;;) {
        uint32_t empty, eod, m = lust_hash_group_(t->ctrl + pos, tag, &empty, &eod);
        while (m) {
            uint64_t index = (pos + (uint64_t)__builtin_ctz(m)) & mask;
            m &= m - 1;
            void *slot = (void *)(t->data + index * t->elem_size);
            if (eq(ctx, slot)) {
                if (out_slot) *out_slot = slot;
                if (out_index) *out_index = index;
                return 1;
            }
        }
        if (!have_insert && eod) {
            insert = (pos + (uint64_t)__builtin_ctz(eod)) & mask;
            have_insert = 1;
        }
        if (have_insert && empty) {
            if (__builtin_expect((t->ctrl[insert] & 0x80u) == 0, 0)) {
                uint32_t e0, d0;
                (void)lust_hash_group_(t->ctrl, 0, &e0, &d0);
                insert = d0 ? (uint64_t)__builtin_ctz(d0) : 0;
            }
            if (out_index) *out_index = insert;
            return 0;
        }
        stride += LUST_HASH_GROUP_WIDTH;
        pos = (pos + stride) & mask;
    }
}
/* lust_hash_insert_at (RawTable::record_item_insert_at): the tag in the bucket's control
   byte and in its mirror, one more item, one less room when the byte was EMPTY. */
static inline void *lust_hash_insert_at_inl_(lust_hash_table *t, uint64_t hash, uint64_t index) {
    uint8_t tag = lust_hash_tag(hash);
    if (t->ctrl[index] == LUST_HASH_EMPTY) t->growth_left -= 1;
    t->ctrl[index] = tag;
    t->ctrl[((index - LUST_HASH_GROUP_WIDTH) & t->bucket_mask) + LUST_HASH_GROUP_WIDTH] = tag;
    t->items += 1;
    return (void *)(t->data + index * t->elem_size);
}
#define lust_hash_find(t, hash, eq, ctx, out) lust_hash_find_inl_((t), (hash), (eq), (ctx), (out))
#define lust_hash_find_or_insert(t, hash, eq, ctx, hash_of, hash_ctx, out_slot, out_index) \
    lust_hash_find_or_insert_inl_((t), (hash), (eq), (ctx), (hash_of), (hash_ctx), (out_slot), (out_index))
#define lust_hash_insert_at(t, hash, index) lust_hash_insert_at_inl_((t), (hash), (index))
#endif

#ifdef LUST_IMPL
#include "hash.c"
#endif

#endif
