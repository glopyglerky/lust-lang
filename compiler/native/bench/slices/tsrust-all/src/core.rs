//! Shared handles and stores for the Go port. Read `PORTING.md` first.
//!
//! Every Go pointer to a shared object becomes a `Copy` handle. Handle value
//! zero is Go `nil`, so Go `x == nil` ports to `x.is_nil()`.

use crate::indexmap::IndexMap;
use crate::rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use std::sync::Arc;

/// `IndexMap` with the Fx hasher. Keys are small ids or short names, so Fx
/// is faster than SipHash. Iteration order is still insertion order.
pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
/// `IndexSet` with the Fx hasher.
pub type FxIndexSet<K> = crate::indexmap::IndexSet<K, FxBuildHasher>;

use crate::flags::{CheckFlags, FlowFlags, NodeFlags, SymbolFlags};

macro_rules! handle {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
        pub struct $name(pub u32);

        impl $name {
            /// Go `nil`.
            pub const NIL: Self = Self(0);

            #[must_use]
            pub const fn is_nil(self) -> bool {
                self.0 == 0
            }

            #[must_use]
            pub const fn is_some(self) -> bool {
                self.0 != 0
            }

            /// Arena index. Arenas keep a dummy entry at index 0.
            #[must_use]
            pub const fn index(self) -> usize {
                self.0 as usize
            }

            /// Returns `None` for nil.
            #[must_use]
            pub const fn get(self) -> Option<Self> {
                if self.0 == 0 { None } else { Some(self) }
            }
        }
    };
}

handle!(
    /// Go `*Type`. Index into `Checker::types`.
    TypeId
);
handle!(
    /// Go `*ast.Symbol`. Index into `SymbolArena::symbols`.
    SymbolId
);
handle!(
    /// Go `*Signature`. Index into `Checker::signatures`.
    SignatureId
);
handle!(
    /// Go `*IndexInfo`. Index into `Checker::index_infos`.
    IndexInfoId
);
handle!(
    /// Go `*TypePredicate`. Index into `Checker::type_predicates`.
    TypePredicateId
);
handle!(
    /// Go `*TypeMapper`. Index into `Checker::mappers`.
    MapperId
);
handle!(
    /// Go `*InferenceContext`. Index into `Checker::inference_contexts`.
    InferenceContextId
);
handle!(
    /// Go `ast.SymbolTable` (a Go map, so it has reference semantics).
    /// Index into `SymbolArena::tables`. Nil is a nil map: reads see it as empty.
    SymbolTable
);

/// Go `*ast.FlowNode`. High 32 bits: file index. Low 32 bits: index + 1 into
/// that file's `GoFile::flow_nodes`. Zero is nil. Read with `flow.get_flow()`
/// (defined in `crate::ast`).
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct FlowNodeId(pub u64);

impl FlowNodeId {
    pub const NIL: Self = Self(0);

    #[must_use]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_some(self) -> bool {
        self.0 != 0
    }

    #[must_use]
    pub const fn new(file: usize, index: usize) -> Self {
        Self(((file as u64) << 32) | (index as u64 + 1))
    }

    #[must_use]
    pub const fn file_index(self) -> usize {
        (self.0 >> 32) as usize
    }

    #[must_use]
    pub const fn local_index(self) -> usize {
        ((self.0 & 0xffff_ffff) - 1) as usize
    }
}

/// Go `*ast.Node` (and every alias: `*ast.Expression`, `*ast.TypeNode`,
/// `*ast.SourceFile`, ...). High 32 bits: file id in the file registry
/// (`ast/store.rs`). For a ported-parser file this is also its store id.
/// Low 32 bits: `crate::astdata::NodeId::index() + 1`. Zero is nil.
/// Node methods (kind, parent, fields, binder data) live in `crate::ast`.
#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Debug)]
pub struct Node(pub u64);

impl Node {
    pub const NIL: Self = Self(0);

    #[must_use]
    pub const fn is_nil(self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_some(self) -> bool {
        self.0 != 0
    }

    #[must_use]
    pub const fn get(self) -> Option<Self> {
        if self.0 == 0 { None } else { Some(self) }
    }



    /// File id in the file registry (`crate::ast::go_file`).
    #[must_use]
    pub const fn file_index(self) -> usize {
        (self.0 >> 32) as usize
    }

    #[must_use]
    pub fn node_id(self) -> crate::astdata::NodeId {
        crate::astdata::NodeId::new(((self.0 & 0xffff_ffff) - 1) as u32)
    }
}

/// A symbol name and symbol table key (Go `string`). Every distinct text is
/// stored once for the whole process (see `intern`), and a `Name` is the
/// 4-byte id of that text. Clones copy the id. Equal names have equal ids,
/// so `==` compares ids. It orders, hashes and prints like the `str` it
/// holds. The low bit of the id is set when the text starts with
/// `INTERNAL_SYMBOL_NAME_PREFIX` (`Name::is_internal`).
#[derive(PartialEq, Eq)]
pub struct Name(u32);

impl Name {

    /// True when the text starts with `INTERNAL_SYMBOL_NAME_PREFIX`.
    /// The id holds this bit, so this reads no text.
    #[inline]
    #[must_use]
    pub fn is_internal(&self) -> bool {
        self.0 & intern::INTERNAL_BIT != 0
    }

    /// True when the text is `INTERNAL_SYMBOL_NAME_DEFAULT` ("default").
    /// That name is interned first, at a fixed id, so this reads no text.
    #[inline]
    #[must_use]
    pub fn is_default_symbol_name(&self) -> bool {
        self.0 == intern::DEFAULT_ID
    }

    /// `table_hash` of the text. The interner keeps it, so this reads one
    /// number and does not touch the text.
    #[inline]
    fn table_hash(&self) -> u32 {
        intern::table_hash(self.0)
    }



    /// The 4-byte id, for a packed node column (`ast::store::NodeKids`).
    /// It is valid only in this process.
    #[inline]
    #[must_use]
    pub(crate) fn id(&self) -> u32 {
        self.0
    }

    /// The name whose `Name::id` in this process is `id`.
    #[inline]
    #[must_use]
    pub(crate) fn from_id(id: u32) -> Name {
        Name(id)
    }
}

// PORT: not `Copy`, so existing `.clone()` calls stay clean for clippy.
impl Clone for Name {
    #[inline]
    fn clone(&self) -> Self {
        Name(self.0)
    }
}

impl Default for Name {
    fn default() -> Self {
        Name(0)
    }
}












impl From<&Name> for Name {
    fn from(s: &Name) -> Self {
        s.clone()
    }
}


impl Name {
}







/// The names of the bundled lib files, made by
/// `crates/ts_goport/scripts/gen-lib-names.py` (`intern::lib_name`).
#[cfg(not(target_family = "wasm"))]
mod lib_names;

/// wasm: an empty lib name table, so a lib name goes through the shards
/// like any other name. The table only makes interning faster. Only the lib
/// snapshots need its stable ids (`Name::stable_id`), and wasm reads no
/// snapshot (`bundled::bundled_lib_name`). No output depends on an id:
/// `Name` hashes and orders by its text, and tables compare ids only for
/// equality.
// PERF: the table is 184 KB of data. Without it the module is 185 KB
// smaller raw, 75 KB smaller with gzip -9 and 62 KB smaller with brotli
// -q 11. `scripts/wasm/bench.mjs` (query core, hono, zod; 10 interleaved
// rounds) put each warm and cold median within -2.1% to +0.9% of the
// module with the table. Packing the names with LZMA and building the
// same table on first use saved only 37 KB gzip, and was not faster.
#[cfg(target_family = "wasm")]
mod lib_names {
    pub(super) const COUNT: usize = 0;
    pub(super) const BUCKET_BITS: u32 = 0;
    pub(super) static TEXT: &str = "";
    pub(super) static OFFSETS: [u32; COUNT + 1] = [0];
    pub(super) static BUCKETS: [u16; (1 << BUCKET_BITS) + 1] = [0, 0];
}

/// The process-wide string interner behind `Name`. Text is copied once into
/// leaked blocks and never freed. A name id is `seq << 1 | internal`, where
/// `seq` is a sequence number that starts at 1 and is close to dense (see
/// `next_id`) and `internal` is `Name::is_internal`. The text tables are
/// indexed by `seq` (`id >> 1`). Id 0 is "". Sequence number 1 is
/// "default" (`DEFAULT_ID`), interned when the shards are made. The names of
/// the bundled lib files (`lib_names`) have the sequence numbers from
/// `LIB_FIRST_SEQ` on, in table order, and are never in the shards.
/// `text` reads without a lock; `intern` takes one shard lock on a miss in
/// the per-thread cache and the lib name table.
mod intern {
    use super::{Name, lib_names};
    use crate::rustc_hash::{FxBuildHasher, FxHashMap};
    use std::cell::{Cell, OnceCell};
    use std::collections::HashMap;
    use std::hash::{BuildHasherDefault, Hasher};
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::{Mutex, OnceLock, PoisonError};

    /// Chunk `k` holds `FIRST_CHUNK << k` ids, so a few chunks cover all ids.
    const FIRST_CHUNK_SHIFT: u32 = 10;
    const CHUNKS: usize = 23;
    const SHARDS: usize = 32;
    const BLOCK: usize = 16 * 1024;
    /// `CACHE` sets. Each set holds `CACHE_WAYS` slots, so the cache holds
    /// 2048 names.
    const CACHE_SETS: usize = 1024;
    const CACHE_WAYS: usize = 2;
    /// Initial capacity of each shard map (at least 32K names in all), so
    /// interning a project rarely grows and rehashes a map under its lock.
    const SHARD_CAPACITY: usize = 1024;

    type Slots = Box<[OnceLock<&'static str>]>;
    static TEXTS: [OnceLock<Slots>; CHUNKS] = [const { OnceLock::new() }; CHUNKS];
    /// The table hash (`super::fold_hash`) of each id's text, in the same
    /// chunks as `TEXTS`. A table lookup by `Name` reads it instead of
    /// reading and hashing the text again.
    static HASHES: [OnceLock<Box<[AtomicU32]>>; CHUNKS] = [const { OnceLock::new() }; CHUNKS];
    /// The next sequence number block. Only `next_id` reads it. Sequence
    /// number 1 is `DEFAULT_ID`, and the lib names come next.
    static NEXT: AtomicU32 = AtomicU32::new(LIB_FIRST_SEQ + lib_names::COUNT as u32);
    /// The sequence number of lib name 0 (`lib_names`).
    const LIB_FIRST_SEQ: u32 = 2;
    /// Sequence numbers that a thread takes from `NEXT` at once.
    const ID_BLOCK: u32 = 64;
    /// Every sequence number is below this, so `seq << 1 | 1` fits in a
    /// `u32`.
    const SEQ_LIMIT: u32 = 1 << 31;
    /// The id bit that `Name::is_internal` reads.
    pub(super) const INTERNAL_BIT: u32 = 1;
    /// The id of `INTERNAL_SYMBOL_NAME_DEFAULT` ("default"): sequence
    /// number 1, not internal. `shards` interns it first.
    // PERF: effect P7-2. `is_static_private_identifier_property` tests for
    // this name with an id compare instead of a text load.
    pub(super) const DEFAULT_ID: u32 = 1 << 1;

    /// Hasher for `u64` keys that already are a `hash_str` hash, so a
    /// lookup does not hash the text again. The halves swap because the
    /// shard index uses the top bits, and the map also reads the top bits.
    #[derive(Default)]
    struct HashIsKey(u64);

    impl Hasher for HashIsKey {
        fn finish(&self) -> u64 {
            self.0
        }

        fn write(&mut self, _: &[u8]) {
            unreachable!("HashIsKey takes only u64 keys");
        }

        fn write_u64(&mut self, hash: u64) {
            self.0 = hash.rotate_left(32);
        }
    }

    struct Shard {
        /// `(text, id)` by the `hash_str` hash of the text.
        ids: HashMap<u64, (&'static str, u32), BuildHasherDefault<HashIsKey>>,
        /// Texts whose hash another text in `ids` already has.
        collisions: FxHashMap<&'static str, u32>,
        /// Unused tail of the current text block.
        free: &'static mut [u8],
    }

    /// A shard lock on its own 128 bytes, so threads that lock neighboring
    /// shards do not share a cache line (x86 fetches lines in pairs).
    #[repr(align(128))]
    struct PaddedShard(Mutex<Shard>);

    static SHARD_LOCKS: OnceLock<[PaddedShard; SHARDS]> = OnceLock::new();

    /// One `CACHE` slot: (hash, id, text). Id 0 marks an empty slot.
    type CacheSlot = Cell<(u64, u32, &'static str)>;

    /// One `CACHE` set: slot 0 is the most recently used. Two 32-byte
    /// slots on one 64-byte line, so a lookup reads one cache line.
    #[repr(align(64))]
    struct CacheSet([CacheSlot; CACHE_WAYS]);

    /// The `CACHE` of one thread (64 KiB).
    type Cache = [CacheSet; CACHE_SETS];


    thread_local! {
        /// 2-way set-associative cache of recent `intern` results, made at
        /// the first `intern` of the thread. A hit needs no borrow flag or
        /// `text` lookup.
        // PERF: two ways in place of one direct-mapped slot, so two hot
        // names with the same set do not evict each other. The cache only
        // holds ids the shards gave out, so it never changes an id.
        // PERF (perfplan4 R1): on the heap, not in the static TLS block.
        // glibc copies that block into each new thread on the thread that
        // starts it, and the cache was 64 KiB of its 99 KiB (`tsc -b` starts
        // 20 to 350 threads, many of which never intern).
        static CACHE: OnceCell<Box<Cache>> = const { OnceCell::new() };

        /// The ids this thread took from `NEXT` and did not use yet:
        /// (next, end).
        static IDS: Cell<(u32, u32)> = const { Cell::new((0, 0)) };
    }

    /// A new sequence number. The name id is `seq << 1 | internal`.
    // PERF: a thread takes `ID_BLOCK` numbers from `NEXT` at once, so
    // threads that intern new names do not contend on one atomic, and the
    // names of one file get close numbers (their `HASHES` and `TEXTS` slots
    // share cache lines). An id only has to be unique: ids already came in
    // thread race order, and no output orders by id (`Name` orders by
    // text). A thread that ends leaves its unused numbers as empty slots.
    fn next_id() -> u32 {
        IDS.with(|ids| {
            let (mut next, mut end) = ids.get();
            if next == end {
                next = NEXT.fetch_add(ID_BLOCK, Ordering::Relaxed);
                // `end` is below `SEQ_LIMIT`, so no id is `u32::MAX`.
                end = next
                    .checked_add(ID_BLOCK)
                    .filter(|&end| end < SEQ_LIMIT)
                    .expect("name id overflow");
            }
            ids.set((next + 1, end));
            next
        })
    }

    /// Fx hash of `s`. Symbol tables use it too.
    #[inline]
    pub(super) fn hash_str(s: &str) -> u64 {
        #[cfg(target_pointer_width = "64")]
        {
            let mut hasher = crate::rustc_hash::FxHasher::default();
            hasher.write(s.as_bytes());
            hasher.finish()
        }
        #[cfg(not(target_pointer_width = "64"))]
        fx_hash_64(s.as_bytes())
    }

    /// `crate::rustc_hash::FxHasher` (2.1.2) of `bytes` as it is on a 64-bit
    /// target. On a 32-bit target (wasm32) FxHasher gives a 32-bit hash,
    /// whose top 32 bits are 0. A shard map (`HashIsKey`) on a 32-bit
    /// target reads only the top 32 bits, so it needs this full hash.
    #[cfg(any(test, not(target_pointer_width = "64")))]
    fn fx_hash_64(bytes: &[u8]) -> u64 {
        const K: u64 = 0xf135_7aea_2e62_a9c5;
        const SEED1: u64 = 0x243f_6a88_85a3_08d3;
        const SEED2: u64 = 0x1319_8a2e_0370_7344;
        const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa409_3822_299f_31d0;
        let multiply_mix = |x: u64, y: u64| {
            let full = u128::from(x).wrapping_mul(u128::from(y));
            (full as u64) ^ ((full >> 64) as u64)
        };
        let word = |b: &[u8]| u64::from_le_bytes(b.try_into().expect("8 bytes"));
        let len = bytes.len();
        let mut s0 = SEED1;
        let mut s1 = SEED2;
        if len <= 16 {
            if len >= 8 {
                s0 ^= word(&bytes[0..8]);
                s1 ^= word(&bytes[len - 8..]);
            } else if len >= 4 {
                s0 ^= u64::from(u32::from_le_bytes(bytes[0..4].try_into().expect("4 bytes")));
                s1 ^= u64::from(u32::from_le_bytes(
                    bytes[len - 4..].try_into().expect("4 bytes"),
                ));
            } else if len > 0 {
                s0 ^= u64::from(bytes[0]);
                s1 ^= (u64::from(bytes[len - 1]) << 8) | u64::from(bytes[len / 2]);
            }
        } else {
            let mut bulk = &bytes[..len - 1];
            while let Some((chunk, rest)) = bulk.split_first_chunk::<16>() {
                let t = multiply_mix(
                    s0 ^ word(&chunk[..8]),
                    PREVENT_TRIVIAL_ZERO_COLLAPSE ^ word(&chunk[8..]),
                );
                s0 = s1;
                s1 = t;
                bulk = rest;
            }
            let suffix = &bytes[len - 16..];
            s0 ^= word(&suffix[0..8]);
            s1 ^= word(&suffix[8..16]);
        }
        // `FxHasher::write` adds the byte hash, and `finish` rotates.
        let hash = multiply_mix(s0, s1) ^ (len as u64);
        hash.wrapping_mul(K).rotate_left(26)
    }

    /// Chunk and slot of `id`, by its sequence number `id >> 1`.
    #[inline]
    fn slot(id: u32) -> (usize, usize) {
        let seq = id >> 1;
        let v = (seq >> FIRST_CHUNK_SHIFT) + 1;
        let k = 31 - v.leading_zeros();
        let start = ((1u32 << k) - 1) << FIRST_CHUNK_SHIFT;
        (k as usize, (seq - start) as usize)
    }

    /// Shard of a text with `hash_str` hash `hash`.
    #[inline]
    fn shard_index(hash: u64) -> usize {
        (hash >> 59) as usize % SHARDS
    }



    /// The text of lib name `i` (`lib_names`).
    #[inline]
    fn lib_text(i: usize) -> &'static str {
        &lib_names::TEXT[lib_names::OFFSETS[i] as usize..lib_names::OFFSETS[i + 1] as usize]
    }

    /// The index of `s` in the lib name table, or `None`. `hash` is
    /// `hash_str(s)`. The result depends only on `s`, so a text in the table
    /// always gets the same id and never reaches the shards.
    // PERF: most probes miss (a name that no lib file has), so a name of
    // another length is skipped with the offsets alone, and byte slices
    // skip the `str` char boundary checks, which read the text.
    #[inline]
    fn find_lib_name(s: &str, hash: u64) -> Option<usize> {
        let bucket = (hash as usize) & ((1usize << lib_names::BUCKET_BITS) - 1);
        let start = lib_names::BUCKETS[bucket] as usize;
        let end = lib_names::BUCKETS[bucket + 1] as usize;
        let text = lib_names::TEXT.as_bytes();
        (start..end).find(|&i| {
            let from = lib_names::OFFSETS[i] as usize;
            let to = lib_names::OFFSETS[i + 1] as usize;
            to - from == s.len() && &text[from..to] == s.as_bytes()
        })
    }







    /// The table hash of the text of name id `id`.
    #[inline]
    pub(super) fn table_hash(id: u32) -> u32 {
        if id == 0 {
            return super::fold_hash(hash_str(""));
        }
        let (chunk, index) = slot(id);
        // Relaxed is enough: a thread gets `id` from `intern_shared` under
        // the shard lock, from `lib_name` after its own store, or through a
        // later handoff, and each orders a store of this hash before this
        // load.
        HASHES[chunk].get().expect("unknown name id")[index].load(Ordering::Relaxed)
    }



    #[cfg(test)]
    mod tests {
        use super::*;

        /// `fx_hash_64` gives the 64-bit FxHasher values of rustc-hash
        /// 2.1.2's own `tests::bytes`, and the 64-bit `hash_str` for texts
        /// of each length class of its byte hash.
        #[test]
        fn fx_hash_64_matches_fx_hasher() {
            for (bytes, want) in [
                (&b""[..], 17_606_491_139_363_777_937),
                (b"\x00", 5_448_590_020_104_574_886),
                (b"\x00\x00\x00\x00\x00\x00", 16_766_921_560_080_789_783),
                (b"\x01", 5_922_447_956_811_044_110),
                (b"\x02", 5_229_781_508_510_959_783),
                (b"uwu", 7_168_164_714_682_931_527),
                (
                    b"These are some bytes for testing rustc_hash.",
                    2_349_210_501_944_688_211,
                ),
            ] {
                assert_eq!(fx_hash_64(bytes), want, "{bytes:?}");
            }
            #[cfg(target_pointer_width = "64")]
            for len in 0..40u32 {
                let text: String = (0..len)
                    .map(|i| char::from(b'a' + (i * 7 % 26) as u8))
                    .collect();
                assert_eq!(fx_hash_64(text.as_bytes()), hash_str(&text), "{text:?}");
            }
        }

        /// Every lib name is found at its own index. A failure after a
        /// rustc-hash update means the `hash_str` copy in
        /// `scripts/gen-lib-names.py` is out of date: fix it and run it.
        #[test]
        fn lib_names_find_themselves() {
            let mut seen = crate::rustc_hash::FxHashSet::default();
            for i in 0..lib_names::COUNT {
                let text = lib_text(i);
                assert!(!text.is_empty() && text.is_ascii(), "{text:?}");
                assert_ne!(text, crate::ast::INTERNAL_SYMBOL_NAME_DEFAULT);
                assert!(seen.insert(text), "lib name {text:?} twice");
                assert_eq!(find_lib_name(text, hash_str(text)), Some(i), "{text:?}");
            }
            assert_eq!(
                lib_names::OFFSETS[lib_names::COUNT] as usize,
                lib_names::TEXT.len()
            );
            assert_eq!(
                lib_names::BUCKETS[1usize << lib_names::BUCKET_BITS] as usize,
                lib_names::COUNT
            );
        }

        /// A lib name gets a table id, a text and a table hash like any other
        /// name, and one id per text.
        #[test]
        fn lib_name_ids() {
            let lib = Name::from("addEventListener");
            assert!(((lib.0 >> 1) - LIB_FIRST_SEQ) < lib_names::COUNT as u32);
            assert!(!lib.is_internal());
            assert_eq!(lib.as_str(), "addEventListener");
            assert_eq!(
                lib.table_hash(),
                super::super::table_hash("addEventListener")
            );
            assert_eq!(Name::from(String::from("addEventListener")).0, lib.0);
            let other = Name::from("notALibName_u5");
            assert!((other.0 >> 1) >= LIB_FIRST_SEQ + lib_names::COUNT as u32);
            assert_eq!(other.as_str(), "notALibName_u5");
            assert!(Name::from("default").is_default_symbol_name());
        }

        /// "", "default" and the lib names have stable ids, and a name made
        /// from a stable id equals the interned name.
        #[test]
        fn stable_ids() {
            for text in ["", "default", "addEventListener"] {
                let name = Name::from(text);
                let id = name.stable_id().expect(text);
                let loaded = Name::from_stable_id(id).expect(text);
                assert_eq!(loaded, name);
                assert_eq!(loaded.table_hash(), super::super::table_hash(text));
            }
            assert_eq!(Name::from("notALibName_u6").stable_id(), None);
            assert!(Name::from_stable_id(u32::MAX).is_none());
        }
    }
}

/// `Symbol::declarations` (Go `[]*ast.Node`). An empty list allocates
/// nothing and a single declaration is stored inline. Longer lists share one
/// `Vec` between clones; the first write copies it, so each symbol still owns
/// its own list, like a Go slice that is copied before an append. Program
/// symbols keep longer lists in a leaked `Vec` (`make_static`), with the same
/// copy on the first write.
// PERF (memper1): every list is behind a thin pointer, so a `Declarations`
// is 16 bytes and a `Symbol` 56 (a leaked `&'static [Node]` made both 8
// bytes larger). A static list reads through its `Vec` header, as a shared
// list reads through its `Arc`.
#[derive(Clone, Default)]
pub struct Declarations(DeclarationList);

#[derive(Clone, Default)]
enum DeclarationList {
    #[default]
    Empty,
    One(Node),
    Many(Arc<Vec<Node>>),
    Static(&'static Vec<Node>),
}

impl Declarations {
    /// Go `append(declarations, node)`.
    pub fn push(&mut self, node: Node) {
        match &mut self.0 {
            DeclarationList::Empty => self.0 = DeclarationList::One(node),
            DeclarationList::One(first) => {
                self.0 = DeclarationList::Many(Arc::new(vec![*first, node]));
            }
            DeclarationList::Many(list) => Arc::make_mut(list).push(node),
            DeclarationList::Static(list) => {
                let mut owned = Vec::with_capacity(list.len() + 1);
                owned.extend_from_slice(*list);
                owned.push(node);
                self.0 = DeclarationList::Many(Arc::new(owned));
            }
        }
    }

    /// Moves a `Many` list into a leaked `Vec`. Call it only for symbols
    /// that live until exit (program symbols, like the AST).
    // PERF: a clone of a `Static` list copies a pointer. A clone of a `Many`
    // list changes a reference count that the checker threads share
    // (`instantiate_symbol`, `clone_symbol`, member code).
    fn make_static(&mut self) {
        if matches!(self.0, DeclarationList::Many(_)) {
            if let DeclarationList::Many(list) = std::mem::take(&mut self.0) {
                self.0 = DeclarationList::Static(Box::leak(Box::new(Arc::unwrap_or_clone(list))));
            }
        }
    }
}




impl From<Vec<Node>> for Declarations {
    fn from(v: Vec<Node>) -> Self {
        Declarations(match v.as_slice() {
            [] => DeclarationList::Empty,
            [node] => DeclarationList::One(*node),
            _ => DeclarationList::Many(Arc::new(v)),
        })
    }
}


/// By-value iterator over `Declarations`. It reads a shared or static list
/// in place.
pub struct DeclarationsIntoIter {
    list: Declarations,
    next: usize,
}






// PERF (memper1): a checker makes millions of symbols, in chunks of
// `COW_CHUNK_LEN` (14 KiB at 56 bytes, a jemalloc size class).

/// A growable array split into fixed-size chunks that clones share. Index
/// `i` is value `i % COW_CHUNK_LEN` of chunk `i / COW_CHUNK_LEN`. The values
/// after the last chunk are the tail, one `Vec` of at most one chunk. A
/// chunk is owned by this array (written without an atomic operation),
/// static (a chunk that lives until exit, `freeze_from`), shared with clones
/// by `Arc` (`share_from`) or freed (`free_chunks`). A clone copies the owned
/// chunks and the tail, and shares the others. The first write to a static
/// or full shared chunk takes it back (a copy, unless no clone uses a shared
/// chunk).
///
/// An owned or static chunk is always full. Only a shared chunk can hold
/// fewer values: the tail that `share_from` or `end_chunk` shared. A write
/// to it keeps it shared, and the next push takes it back as the tail.
/// So a read at or above the length panics (index out of bounds), as a
/// `Vec` read does.
///
/// Holes (lsshells M3d): `end_chunk` moves the length to the next chunk
/// start, so the indexes left in the last chunk hold no value, and
/// `free_chunks` frees whole chunks. A hole is in a shared chunk that holds
/// only the values before it, or in a freed chunk, so a read of a hole
/// panics. Indexes never move and are never used again.
///
/// Interleaved arrays (apisym1c, `new_interleaved`): the binder lineage
/// puts its values in the even chunks only, and a checker copy of it
/// (`for_checker`) puts its own values in the odd chunks only. A sealed
/// chunk is followed by a freed chunk, so the tail keeps its parity. So a
/// later lineage index is never one of a checker's own, and a checker can
/// add the chunks that the lineage bound after its copy (`catch_up`), as a
/// Go checker reads every bound file. In a checker copy, the lineage chunk
/// that held the copy's last values is an owned chunk padded with default
/// values: its indexes after those values read as default, not as holes.
// PERF: an owned or static chunk is a fixed array behind a thin pointer at
// the same place in the chunk entry, so `get` of one is one bounds test (the
// chunk list), one kind test and one load, with no branch between the two
// kinds and no pointer hop. A read of the tail is one bounds test more (its
// `Vec`), and that test also stops a read at or above the length. Shared and
// freed chunks (only in the language service) go out of line.
// PERF (apisym1c): the parity is in the index, so `get` does not select a
// part (a second array cost 0.8 to 2.5 percent instructions in a check).
#[derive(Clone, Debug)]
pub struct CowChunks<T: 'static> {
    chunks: Vec<Chunk<T>>,
    /// The values from index `chunks.len() * COW_CHUNK_LEN` on. Empty, with
    /// no capacity, while the length is inside the last chunk (a partial
    /// shared chunk).
    tail: Vec<T>,
    len: usize,
    /// Each sealed chunk is followed by a freed chunk (`new_interleaved`).
    interleaved: bool,
}

const COW_CHUNK_SHIFT: usize = 8;
const COW_CHUNK_LEN: usize = 1 << COW_CHUNK_SHIFT;
const COW_CHUNK_MASK: usize = COW_CHUNK_LEN - 1;

type ChunkValues<T> = [T; COW_CHUNK_LEN];

#[derive(Clone, Debug)]
enum Chunk<T: 'static> {
    /// Read in place (`get` tests for this kind once). Always full.
    Fixed(Fixed<T>),
    /// Only the first `len()` indexes hold values.
    Shared(Arc<Vec<T>>),
    Freed,
}

// PERF: the two kinds keep the pointer at the same place, so `get` reads
// both with one load and no branch.
#[derive(Clone, Debug)]
enum Fixed<T: 'static> {
    Owned(Box<ChunkValues<T>>),
    /// A leaked full chunk (`CowChunks::freeze_from`).
    Static(&'static ChunkValues<T>),
}


impl<T> Fixed<T> {
}

impl<T: Clone> Chunk<T> {



    fn has_values(&self) -> bool {
        match self {
            Chunk::Fixed(_) => true,
            Chunk::Shared(values) => !values.is_empty(),
            Chunk::Freed => false,
        }
    }
}

impl<T: Clone> CowChunks<T> {


    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// The index that the next push gets: the length, or in an interleaved
    /// array whose tail is full, the start of the chunk after the freed
    /// chunk that the push puts after the tail (`seal_full_tail`).
    #[must_use]
    pub fn next_index(&self) -> usize {
        if self.interleaved && self.tail.len() == COW_CHUNK_LEN {
            self.len + COW_CHUNK_LEN
        } else {
            self.len
        }
    }

    /// The number of indexes below `next_index` that the array's parity
    /// holds: in an interleaved binder array, the lineage indexes (even
    /// chunks), as `ArenaOffsets` counts them; else `next_index`.
    #[must_use]
    pub fn dense_len(&self) -> usize {
        let next = self.next_index();
        if self.interleaved {
            next - ((next >> (COW_CHUNK_SHIFT + 1)) << COW_CHUNK_SHIFT)
        } else {
            next
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The index of the first tail value.
    #[inline(always)]
    fn tail_start(&self) -> usize {
        self.chunks.len() << COW_CHUNK_SHIFT
    }

    // PERF: the value is written into the tail inline when it has room, and
    // the out-of-line `make_room` takes no value. So an inlined caller builds
    // the value in its slot and does not copy it from the stack.


    /// Moves the values of the last chunk back into the tail: a partial
    /// shared chunk, which holds the length (a copy when a clone still uses
    /// it).
    fn take_back_tail(&mut self) {
        debug_assert!(self.tail.is_empty(), "a tail after a partial chunk");
        self.tail = match self.chunks.pop() {
            Some(Chunk::Shared(values)) => Arc::unwrap_or_clone(values),
            _ => panic!("write to a freed chunk"),
        };
    }


    /// Makes room for `additional` more values, so the pushes that follow
    /// do not grow step by step: in the tail up to one chunk while the array
    /// has no chunk, and in the chunk list when the values need more than
    /// one chunk. Capacity only.
    // PERF: bind D. A file arena started with a one-value first chunk,
    // which `push` grew by doubling for every bound file. The tail gets at
    // most one chunk of room: the chunks after it free one by one in
    // `into_aligned`, so a large file arena (a lib snapshot load) does not
    // hold its whole buffer while its values move.
    pub fn reserve(&mut self, additional: usize) {
        if self.chunks.is_empty() {
            let room = COW_CHUNK_LEN.saturating_sub(self.tail.len());
            self.tail.reserve_exact(additional.min(room));
        }
        if self.len + additional > COW_CHUNK_LEN {
            let chunks = (self.len + additional) >> COW_CHUNK_SHIFT;
            self.chunks
                .reserve(chunks.saturating_sub(self.chunks.len()));
        }
    }


    /// `get` from a shared or freed chunk.
    #[cold]
    #[inline(never)]
    fn get_other(&self, i: usize) -> &T {
        match &self.chunks[i >> COW_CHUNK_SHIFT] {
            Chunk::Shared(values) => &values[i & COW_CHUNK_MASK],
            _ => panic!("read of freed index {i}"),
        }
    }






    /// The number of chunks that hold values (freed chunks do not count),
    /// with the tail.
    #[must_use]
    pub fn live_chunks(&self) -> usize {
        usize::from(!self.tail.is_empty())
            + self
                .chunks
                .iter()
                .filter(|chunk| chunk.has_values())
                .count()
    }




}


// apisym1c: the checker copies of the binder lineage.
impl<T: Clone + Default> CowChunks<T> {


}

/// Values moved out of a `CowChunks` for `CowChunks::append_aligned` at
/// index `at`: the values that fill the partial tail at `at`, then whole
/// chunks and the values after them (the new tail).
#[derive(Debug)]
pub struct AlignedChunks<T: 'static> {
    at: usize,
    len: usize,
    head: Vec<T>,
    chunks: Vec<Chunk<T>>,
    tail: Vec<T>,
}


// PORT: no Go counterpart. Holes of the binder lineage (lsshells M3d).
#[cfg(test)]
mod hole_tests {
    use super::*;

    /// Pushes each index as its value until the length is `end`.
    fn push_to(values: &mut CowChunks<usize>, end: usize) {
        while values.len() < end {
            let index = values.len();
            values.push(index);
        }
    }

    // A range that `end_chunk` starts and ends is whole chunks. Freeing it
    // makes its indexes (and the padding before it) holes that panic on a
    // read, keeps every other index, and a clone keeps the freed values.
    #[test]
    fn freed_chunks_are_holes_and_other_indexes_stay() {
        let mut values = CowChunks::new();
        push_to(&mut values, 300);
        values.end_chunk();
        let start = values.len();
        assert_eq!(start, 2 * COW_CHUNK_LEN);
        push_to(&mut values, start + 600);
        values.end_chunk();
        let end = values.len();
        assert_eq!(end, 5 * COW_CHUNK_LEN);
        push_to(&mut values, end + 10);
        values.share_from(0);
        let clone = values.clone();
        assert_eq!(values.live_chunks(), 6);

        values.free_chunks(start, end);
        assert_eq!(values.live_chunks(), 3);
        assert_eq!((*values.get(299), *values.get(end + 9)), (299, end + 9));
        let read = |index: usize| std::panic::catch_unwind(|| *values.get(index)).is_err();
        assert!(read(start + 1), "a read of a freed index panics");
        assert!(read(400), "a read of the padding panics");
        assert_eq!(
            *clone.get(start + 1),
            start + 1,
            "the clone keeps its chunks"
        );
        assert_eq!(clone.live_chunks(), 6);

        values.share_from(0);
        assert_eq!(values.live_chunks(), 3, "a freed chunk stays empty");
        values.push(end + 10);
        assert_eq!((values.len(), *values.get(end + 10)), (end + 11, end + 10));
    }

    // A range that pushes filled to its last index is freed too: there
    // `end_chunk` does nothing, so the range's last chunk is still the tail
    // when no `share_from` came between (gaps147 skeptic, cowfuzz latent2).
    #[test]
    fn a_full_tail_in_the_range_is_freed() {
        let mut values = CowChunks::new();
        push_to(&mut values, 300);
        values.end_chunk();
        let start = values.len();
        push_to(&mut values, start + COW_CHUNK_LEN);
        values.end_chunk();
        let end = values.len();
        assert_eq!((start, end), (2 * COW_CHUNK_LEN, 3 * COW_CHUNK_LEN));
        assert_eq!(values.live_chunks(), 3);

        values.free_chunks(start, end);
        assert_eq!(values.live_chunks(), 2);
        let read = |index: usize| std::panic::catch_unwind(|| *values.get(index)).is_err();
        assert!(read(start + 5), "a read of a freed index panics");
        assert_eq!(*values.get(299), 299);
        values.push(end);
        assert_eq!((values.len(), *values.get(end)), (end + 1, end));
    }

    // A read at or above the length panics, as a `Vec` read does: in the
    // tail with no chunk, in the tail after chunks, in a partial shared
    // chunk that a write took back, and after a static chunk.
    #[test]
    fn reads_at_or_above_the_length_panic() {
        let read = |values: &CowChunks<usize>, index: usize| {
            std::panic::catch_unwind(|| *values.get(index)).is_err()
        };
        let mut values = CowChunks::new();
        push_to(&mut values, 10);
        assert!(!read(&values, 9) && read(&values, 10), "no chunk");
        push_to(&mut values, 300);
        assert!(!read(&values, 299) && read(&values, 300) && read(&values, 511));

        values.share_from(0);
        *values.get_mut(260) = 1;
        assert!(
            read(&values, 300),
            "a written partial chunk keeps its holes"
        );
        values.push(300);
        assert_eq!((*values.get(260), *values.get(300)), (1, 300));
        assert!(read(&values, 301));

        push_to(&mut values, 600);
        values.freeze_from(0);
        assert!(!read(&values, 599) && read(&values, 600) && read(&values, 767));
        *values.get_mut(10) = 2;
        assert_eq!((*values.get(10), values.len()), (2, 600));
    }

    // The values of a freed chunk drop with the last clone that shares it.
    #[test]
    fn freed_values_drop_with_the_last_clone() {
        let token = Arc::new(());
        let mut values = CowChunks::new();
        for _ in 0..10 {
            values.push(Arc::clone(&token));
        }
        values.end_chunk();
        values.share_from(0);
        let clone = values.clone();
        values.free_chunks(0, COW_CHUNK_LEN);
        assert_eq!(Arc::strong_count(&token), 11);
        drop(clone);
        assert_eq!(Arc::strong_count(&token), 1);
    }

    // A file arena joins at aligned offsets as it would bind there: the
    // offsets are `ArenaOffsets::aligned`, its ids start at a chunk start
    // (the next even chunk of the interleaved lineage), and its tables
    // point to its symbols. Freeing its range keeps the symbols before it.
    #[test]
    fn file_arena_joins_at_a_chunk_start() {
        let mut arena = SymbolArena::new();
        let before = arena.new_symbol(SymbolFlags::NONE, "before");
        for _ in 0..299 {
            arena.new_symbol(SymbolFlags::NONE, "other");
        }
        let offsets = arena.next_file_offsets();
        arena.end_chunk();
        assert_eq!(arena.next_file_offsets(), offsets.aligned());
        assert_eq!(offsets.aligned().aligned(), offsets.aligned());

        let mut file = SymbolArena::new_file();
        let symbol = file.new_symbol(SymbolFlags::NONE, "f");
        let members = file.new_table();
        file.set(members, "f", symbol);
        file.sym_mut(symbol).members = members;
        let start = arena.mark();
        let moved = arena.append_file_arena(file, true);
        assert_eq!(moved, offsets.aligned());
        let id = moved.symbol(symbol);
        assert_eq!(id.index(), 4 * COW_CHUNK_LEN);
        assert_eq!(arena.sym(id).name.as_str(), "f");
        assert_eq!(arena.get(arena.sym(id).members, "f"), id);

        arena.end_chunk();
        let live = arena.live_chunk_count();
        arena.free_range(start, arena.mark());
        assert_eq!(arena.live_chunk_count(), live - 2);
        assert_eq!(arena.sym(before).name.as_str(), "before");
    }

    /// Pushes `count` values, each its own index, and returns the indexes.
    fn push_indexes(values: &mut CowChunks<usize>, count: usize) -> Vec<usize> {
        (0..count)
            .map(|_| {
                let index = values.next_index();
                values.push(index);
                assert_eq!(values.len() - 1, index, "next_index");
                index
            })
            .collect()
    }

    // apisym1c: an interleaved array puts its values in the even chunks, a
    // freed chunk after each, so a read of an odd index panics. `end_chunk`
    // and a full tail move to the next even chunk, and `dense_len` counts
    // the even indexes.
    #[test]
    fn an_interleaved_array_skips_the_odd_chunks() {
        let mut values = CowChunks::new_interleaved();
        let pushed = push_indexes(&mut values, 600);
        let chunk = |index: usize| index >> COW_CHUNK_SHIFT;
        assert!(pushed.iter().all(|&index| chunk(index) % 2 == 0));
        assert_eq!(
            (pushed[255], pushed[256], pushed[599]),
            (255, 2 * COW_CHUNK_LEN, 4 * COW_CHUNK_LEN + 87)
        );
        assert!(pushed.iter().all(|&index| *values.get(index) == index));
        let read = |values: &CowChunks<usize>, index: usize| {
            std::panic::catch_unwind(|| *values.get(index)).is_err()
        };
        assert!(read(&values, COW_CHUNK_LEN) && read(&values, 3 * COW_CHUNK_LEN + 1));
        assert_eq!(values.dense_len(), 600);
        values.end_chunk();
        assert_eq!((values.len(), values.dense_len()), (6 * COW_CHUNK_LEN, 768));
        push_indexes(&mut values, COW_CHUNK_LEN);
        assert_eq!(values.next_index(), 8 * COW_CHUNK_LEN, "a full tail");
        assert_eq!(values.dense_len(), 4 * COW_CHUNK_LEN);
        values.end_chunk();
        assert_eq!(values.len(), 8 * COW_CHUNK_LEN);
        values.free_chunks(6 * COW_CHUNK_LEN, 8 * COW_CHUNK_LEN);
        assert!(read(&values, 6 * COW_CHUNK_LEN) && !read(&values, pushed[599]));
    }

    // apisym1c: a checker copy of the lineage makes its own values in the
    // odd chunks and catches up to later lineage values. It keeps what it
    // wrote and its own values, takes the lineage chunks bound since (a
    // partial one padded), ends its tail when the lineage passes it, and
    // catches up again inside a chunk.
    #[test]
    fn a_checker_copy_catches_up_to_later_lineage_values() {
        let mut lineage = CowChunks::new_interleaved();
        let mut bound = push_indexes(&mut lineage, 300);
        lineage.freeze_from(0);
        let mut checker = lineage.for_checker();
        let seen = lineage.len();
        let own: Vec<usize> = (0..300)
            .map(|n| {
                checker.push(10_000 + n);
                checker.len() - 1
            })
            .collect();
        assert!(own.iter().all(|&index| (index >> COW_CHUNK_SHIFT) % 2 == 1));
        assert_eq!(own[0], 3 * COW_CHUNK_LEN, "after the lineage's tail chunk");
        assert_eq!(checker.odd_indexes().collect::<Vec<_>>(), own);
        *checker.get_mut(5) = 1005;
        *checker.get_mut(bound[290]) = 1290;

        // A static file fills the tail chunk and two more; then a freeable
        // version on chunks of its own, and the tail of a static file.
        bound.extend(push_indexes(&mut lineage, 600));
        lineage.freeze_from(0);
        lineage.end_chunk();
        let start = lineage.len();
        let version = push_indexes(&mut lineage, 10);
        lineage.end_chunk();
        lineage.share_from(start);
        bound.extend(push_indexes(&mut lineage, 3));
        checker.catch_up(seen, &lineage);
        for &index in bound.iter().chain(&version) {
            let expected = match index {
                5 => 1005,
                _ if index == bound[290] => 1290,
                _ => index,
            };
            assert_eq!(*checker.get(index), expected, "{index}");
        }
        for (n, &index) in own.iter().enumerate() {
            assert_eq!(*checker.get(index), 10_000 + n, "own {index}");
        }
        // The lineage passed the own tail (chunk 5): the next own value
        // starts after the lineage's last chunk.
        checker.push(20_000);
        let next = checker.len() - 1;
        assert_eq!(
            next >> COW_CHUNK_SHIFT,
            (lineage.len() >> COW_CHUNK_SHIFT) + 1
        );
        let mut all_own = own.clone();
        all_own.push(next);
        assert_eq!(checker.odd_indexes().collect::<Vec<_>>(), all_own);

        // More values in the lineage's tail chunk: a catch-up inside it.
        let seen = lineage.len();
        let more = push_indexes(&mut lineage, 5);
        checker.catch_up(seen, &lineage);
        assert!(more.iter().all(|&index| *checker.get(index) == index));
        assert_eq!(*checker.get(next), 20_000);

        // The version dies: its chunks go, the own chunks stay.
        checker.free_chunks(start, start + 2 * COW_CHUNK_LEN);
        let read = |index: usize| std::panic::catch_unwind(|| *checker.get(index)).is_err();
        assert!(read(version[0]));
        assert_eq!(*checker.get(own[299]), 10_299);
    }

    // followups31 (R177 reviewer item 4): when a catch-up passes the own
    // tail, the tail becomes a shared chunk with its values only, not the
    // room of a whole chunk that it had for later values.
    #[test]
    fn a_catch_up_past_the_own_tail_keeps_only_its_values() {
        let mut lineage = CowChunks::new_interleaved();
        push_indexes(&mut lineage, 10);
        let mut checker = lineage.for_checker();
        let seen = lineage.len();
        let own: Vec<usize> = (0..44)
            .map(|n| {
                checker.push(10_000 + n);
                checker.len() - 1
            })
            .collect();
        let own_chunk = own[0] >> COW_CHUNK_SHIFT;
        assert_eq!(checker.tail.capacity(), COW_CHUNK_LEN, "room for a chunk");
        let later = push_indexes(&mut lineage, 3 * COW_CHUNK_LEN);
        checker.catch_up(seen, &lineage);
        match &checker.chunks[own_chunk] {
            Chunk::Shared(values) => assert_eq!((values.len(), values.capacity()), (44, 44)),
            _ => panic!("the own tail is a shared chunk"),
        }
        for (n, &index) in own.iter().enumerate() {
            assert_eq!(*checker.get(index), 10_000 + n, "own {index}");
        }
        assert!(later.iter().all(|&index| *checker.get(index) == index));
    }

    // apisym1c: a checker arena makes its own symbols and tables in the odd
    // chunks. It catches up to the lineage: it reads a later binder symbol,
    // its table and its members by the lineage ids, keeps its own symbols,
    // and frees the ranges that the lineage freed.
    #[test]
    fn a_checker_arena_catches_up_to_the_lineage() {
        let mut lineage = SymbolArena::new();
        let early = lineage.new_symbol(SymbolFlags::NONE, "early");
        let mut checker = lineage.for_checker();
        let own = checker.new_symbol(SymbolFlags::TRANSIENT, "own");
        let own_table = checker.new_table();
        checker.set(own_table, "early", early);
        assert!(is_own_index(own.0) && is_own_index(own_table.0));
        assert_eq!(checker.symbol_count(), 2);
        assert_eq!(checker.own_symbols().collect::<Vec<_>>(), [own]);

        // A freeable version binds after the copy, then dies.
        lineage.end_chunk();
        let start = lineage.mark();
        let later = lineage.new_symbol(SymbolFlags::NONE, "later");
        let members = lineage.new_table();
        lineage.set(members, "m", later);
        lineage.sym_mut(later).members = members;
        lineage.end_chunk();
        lineage.share_since(start);
        let end = lineage.mark();
        lineage.set_lineage_seen(LineageSeen {
            generation: 1,
            freed: 0,
        });
        checker.catch_up(&lineage, &[]);
        assert_eq!(checker.sym(later).name.as_str(), "later");
        assert_eq!(checker.get(checker.sym(later).members, "m"), later);
        assert_eq!(checker.sym(own).name.as_str(), "own");
        assert_eq!(checker.get(own_table, "early"), early);
        assert_eq!(
            checker.id_slot(later),
            IdSlot {
                key: 0,
                place: later.0
            }
        );
        assert_eq!(checker.symbol_at_slot(checker.id_slot(own)), Some(own));
        assert_eq!(checker.own_symbols().collect::<Vec<_>>(), [own]);
        let live = checker.live_chunk_count();

        lineage.free_range(start, end);
        let freed = [(start, end)];
        lineage.set_lineage_seen(LineageSeen {
            generation: 2,
            freed: 1,
        });
        checker.catch_up(&lineage, &freed);
        assert_eq!(checker.lineage_seen().freed, 1);
        assert_eq!(checker.live_chunk_count(), live - 2);
        assert!(std::panic::catch_unwind(|| checker.sym(later).flags).is_err());
        assert_eq!(checker.sym(early).name.as_str(), "early");
        assert_eq!(checker.sym(own).name.as_str(), "own");
    }
}


/// One symbol table entry: a name id and its symbol. The table hash of the
/// name is in the interner (`TableEntry::hash`), not here.
// PERF (memper1 B): 8 bytes, not 12 with the hash. A lookup by `Name` (most
// lookups) compares ids. A lookup by text and a reindex read the interner's
// hash of each entry they compare or index. One-byte index slots for tables
// of up to 255 entries (half the index) were measured and dropped: the slot
// width test kept the lookup and insert loops out of line (+1.6%
// instructions on zod, single-threaded).
#[derive(Clone, Copy, Debug)]
struct TableEntry {
    name: u32,
    symbol: SymbolId,
}

const _: () = assert!(std::mem::size_of::<TableEntry>() == 8);

impl TableEntry {
    /// The table hash of the name.
    #[inline]
    fn hash(&self) -> u32 {
        intern::table_hash(self.name)
    }
}

/// Tables up to this size are searched linearly and have no index.
const TABLE_LINEAR_MAX: usize = 8;


/// The bit of a table hash in `Table::filter`: its top 6 bits, which the
/// index slot (the low bits) does not use.
#[inline]
fn filter_bit(hash: u32) -> u64 {
    1 << (hash >> 26)
}

const INDEX_MOD: usize = u16::MAX as usize;

/// The table hash of `name`. `Name::table_hash` gives the same value
/// without hashing.
#[inline]
fn table_hash(name: &str) -> u32 {
    fold_hash(intern::hash_str(name))
}

/// Folds an `intern::hash_str` hash to the 32-bit table hash.
#[inline]
fn fold_hash(hash: u64) -> u32 {
    (hash ^ (hash >> 32)) as u32
}

thread_local! {
    /// (address, length, table hash) of the text of the innermost
    /// `with_text_hash` call on this thread. Zeros when there is none: a
    /// `str` address is never 0.
    static TEXT_HASH: std::cell::Cell<(usize, usize, u32)> =
        const { std::cell::Cell::new((0, 0, 0)) };
}


/// The table hash of `text`, from the innermost `with_text_hash` when it
/// covers this exact text.
#[inline]
fn lookup_hash(text: &str) -> u32 {
    let (address, len, hash) = TEXT_HASH.get();
    if address == text.as_ptr().addr() && len == text.len() {
        hash
    } else {
        table_hash(text)
    }
}







/// A state of the binder lineage (`program.rs`): its generation, which each
/// bind and free makes new, and the number of ranges that it has freed. A
/// copy of the lineage keeps the state that it copies, so a checker knows
/// when it must catch up (`SymbolArena::catch_up`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LineageSeen {
    pub generation: u64,
    pub freed: usize,
}

/// Whether symbol or table index `index` of a checker arena is one that the
/// checker made (an odd chunk, see `SymbolArena`), not a lineage index.
#[inline(always)]
#[must_use]
pub const fn is_own_index(index: u32) -> bool {
    index as usize & COW_CHUNK_LEN != 0
}

/// Where `crate::ast::get_symbol_id` keeps the ids of the symbols of one
/// arena. Every binder arena of the process (the binder lineage in
/// `program.rs` and its copies) gives a symbol one id, like Go, where a
/// bound file keeps its symbols in every program. A checker arena
/// (`SymbolArena::for_checker`) makes its own symbols (`is_own_index`),
/// which other checkers do not have, so those symbols have ids of their
/// own, kept by `key`.
#[derive(Debug)]
struct SymbolIds {
    /// `COW_CHUNK_LEN`, the index bit of an own symbol, in a checker arena;
    /// 0 in a binder arena, whose symbols all have shared ids.
    own_bit: u32,
    /// The first own chunk of a checker arena, as a chunk pair (chunk
    /// `2 * own_base + 1`): own symbol places count from it (`own_place`).
    own_base: u32,
    /// The lineage symbol and table lengths that a checker arena copies
    /// (`catch_up` moves them); `u32::MAX` in a binder arena.
    lineage_symbols: u32,
    lineage_tables: u32,
    /// Keys the ids of the own symbols. 0 in a binder arena.
    key: u32,
    /// The shadows in this arena (`SymbolArena::push_shadow`), if any.
    shadows: Option<Box<Shadows>>,
}

/// The shadows of one checker arena: copies of symbols of other arenas.
/// A shadow keeps its id in the slot of its origin.
#[derive(Debug, Default)]
struct Shadows {
    /// The id slot of each shadow's origin, by shadow index.
    origins: FxHashMap<u32, IdSlot>,
    /// The shadow of each origin.
    by_origin: FxHashMap<IdSlot, SymbolId>,
}

impl SymbolIds {
    /// Every symbol has the shared id of its index.
    const SHARED: SymbolIds = SymbolIds {
        own_bit: 0,
        own_base: 0,
        lineage_symbols: u32::MAX,
        lineage_tables: u32::MAX,
        key: 0,
        shadows: None,
    };

    /// The own symbols (from chunk pair `own_base` on) have ids of their
    /// own, under a new key.
    fn own(own_base: u32, lineage_symbols: usize, lineage_tables: usize) -> SymbolIds {
        static NEXT_KEY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);
        SymbolIds {
            own_bit: COW_CHUNK_LEN as u32,
            own_base,
            lineage_symbols: u32::try_from(lineage_symbols).expect("symbol overflow"),
            lineage_tables: u32::try_from(lineage_tables).expect("table overflow"),
            key: NEXT_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            shadows: None,
        }
    }

    /// The place of own symbol index `index` among the own symbols: its
    /// index without the even chunks, from chunk pair `own_base` on. The
    /// places are dense until the first catch-up that passes the own tail
    /// (`CowChunks::catch_up`): the next own symbol then starts after the
    /// lineage, so each odd chunk that the lineage passed is a gap of
    /// `COW_CHUNK_LEN` places, 2 KiB in the own id table once a later own
    /// symbol has an id (`ast::utilities_p1::OwnSymbolIds`).
    #[inline]
    fn own_place(&self, index: u32) -> u32 {
        (((index >> (COW_CHUNK_SHIFT + 1)) - self.own_base) << COW_CHUNK_SHIFT)
            | (index & COW_CHUNK_MASK as u32)
    }

    /// The own symbol index of place `place` (`own_place`).
    fn own_index(&self, place: u32) -> u32 {
        (((place >> COW_CHUNK_SHIFT) + self.own_base) << (COW_CHUNK_SHIFT + 1))
            | self.own_bit
            | (place & COW_CHUNK_MASK as u32)
    }
}

impl Clone for SymbolIds {
    /// A copy of a checker arena holds copies of its own symbols, which are
    /// other symbols, so they get new ids. Copies of shadows too.
    fn clone(&self) -> Self {
        if self.key == 0 {
            SymbolIds::SHARED
        } else {
            SymbolIds::own(
                self.own_base,
                self.lineage_symbols as usize,
                self.lineage_tables as usize,
            )
        }
    }
}


/// Where `crate::ast::get_symbol_id` keeps the id of one symbol
/// (`SymbolArena::id_slot`). Go keeps the id on the `*ast.Symbol`, so a
/// slot stands for one Go symbol: two symbols with one slot, in any
/// arenas, are the same Go symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IdSlot {
    /// 0 for a binder lineage symbol, whose id every arena shares. Else the
    /// key of the checker arena that made the symbol.
    pub key: u32,
    /// The lineage index (key 0), or the place of the symbol among the own
    /// symbols of that arena.
    pub place: u32,
}



/// Where a name is, or goes, in one symbol table (`SymbolArena::get_slot`).
#[derive(Clone, Copy, Debug)]
pub struct TableSlot {
    table: SymbolTable,
    hash: u32,
    name: u32,
    position: Option<usize>,
}

/// Symbol and table counts of a `SymbolArena` at one point
/// (`SymbolArena::mark`).
#[derive(Clone, Copy, Debug)]
pub struct ArenaMark {
    symbols: usize,
    tables: usize,
}

/// Prefix of private identifier symbol names (`<prefix>#<id>@<description>`):
/// `ast::INTERNAL_SYMBOL_NAME_PREFIX` + "#".
const PRIVATE_PREFIX: &str = "\u{FDD0}\u{10F7FE}#";


/// How the ids of a file arena moved in `SymbolArena::append_file_arena`.
/// An offset counts the lineage entries before the file, without the nil
/// entry and the odd chunks (`CowChunks::dense_len`), so a file arena id
/// plus its offset is a dense lineage place (`lineage_index`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaOffsets {
    symbols: u32,
    tables: u32,
}

/// The lineage index of dense lineage place `place`: chunk `n` of the
/// places is the even chunk `2 * n` of the interleaved lineage
/// (`SymbolArena`).
#[inline]
const fn lineage_index(place: u32) -> u32 {
    place + (place & !(COW_CHUNK_MASK as u32))
}

impl ArenaOffsets {

    /// The offsets of the file after a file with these offsets, where
    /// `file_arena` is the `SymbolArena::mark` of that file's own arena.
    /// Parallel binding adds the counts of the earlier files this way.
    #[must_use]
    pub fn after(self, file_arena: ArenaMark) -> ArenaOffsets {
        let count = |len: usize| u32::try_from(len - 1).expect("arena overflow");
        ArenaOffsets {
            symbols: self
                .symbols
                .checked_add(count(file_arena.symbols))
                .expect("symbol overflow"),
            tables: self
                .tables
                .checked_add(count(file_arena.tables))
                .expect("table overflow"),
        }
    }

    /// The program id of file arena symbol `symbol`.
    #[must_use]
    pub fn symbol(self, symbol: SymbolId) -> SymbolId {
        if symbol.is_nil() {
            symbol
        } else {
            SymbolId(lineage_index(symbol.0 + self.symbols))
        }
    }

    /// The program id of file arena table `table`.
    #[must_use]
    pub fn table(self, table: SymbolTable) -> SymbolTable {
        if table.is_nil() {
            table
        } else {
            SymbolTable(lineage_index(table.0 + self.tables))
        }
    }

}

/// Go `*ast.FlowNode`. `antecedents` replaces the Go `FlowList` linked list
/// in the same order.
#[derive(Clone, Debug, Default)]
pub struct FlowNode {
    pub flags: FlowFlags,
    pub node: Node,
    pub antecedent: FlowNodeId,
    pub antecedents: Vec<FlowNodeId>,
}

/// Go `ast.PatternAmbientModule`.
#[derive(Clone, Debug)]
pub struct PatternAmbientModule {
    pub pattern_prefix: String,
    pub pattern_suffix: String,
    pub symbol: SymbolId,
}

/// Binder output for one node. Go stores these on the node itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeBindData {
    pub symbol: SymbolId,
    pub local_symbol: SymbolId,
    pub locals: SymbolTable,
    pub next_container: Node,
    pub flow_node: FlowNodeId,
    pub end_flow_node: FlowNodeId,
    pub return_flow_node: FlowNodeId,
    /// Go `NodeFlags` bits the binder ORs into `node.Flags`.
    pub added_flags: NodeFlags,
}

/// AST node records, step 2: the binder fields of one node that are not in
/// its `NodeRecord` (`ast/store.rs`, which holds the symbol, the added
/// flags and, for a node with no entry here, the flow node). Most nodes
/// have none of them: only locals containers, exported declarations and
/// function-like nodes do.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NodeBindExtra {
    pub local_symbol: SymbolId,
    pub locals: SymbolTable,
    pub next_container: Node,
    pub end_flow_node: FlowNodeId,
    pub return_flow_node: FlowNodeId,
    /// Go `FlowNodeData().FlowNode` of the node: astmem1 P3, its record
    /// has the extras index in place of the flow node (`ast::BIND_EXTRA`).
    /// The install sets it (`ast::bind_store_records`); `of` leaves it nil.
    pub flow_node: FlowNodeId,
}

impl NodeBindExtra {
    /// The fields of `data` that are not in the record, with a nil flow
    /// node.
    #[inline]
    #[must_use]
    pub fn of(data: &NodeBindData) -> Self {
        NodeBindExtra {
            local_symbol: data.local_symbol,
            locals: data.locals,
            next_container: data.next_container,
            end_flow_node: data.end_flow_node,
            return_flow_node: data.return_flow_node,
            flow_node: FlowNodeId::NIL,
        }
    }

    /// The empty fields: a node with no extras entry.
    pub const NONE: Self = NodeBindExtra {
        local_symbol: SymbolId::NIL,
        locals: SymbolTable::NIL,
        next_container: Node::NIL,
        end_flow_node: FlowNodeId::NIL,
        return_flow_node: FlowNodeId::NIL,
        flow_node: FlowNodeId::NIL,
    };
}



/// The binder data of every node of one file, stored compactly, as the lib
/// bind snapshot keeps it (`parts`, `binder::BoundNodes`). Most nodes have
/// no data, or the same data as the node before them (a run of identifiers
/// in one flow region), so they share one entry. Each node keeps one byte:
/// the offset of its entry from the first entry of its block. The install
/// writes it into the node records and a `FileNodeBind`
/// (`ast::bind_store_records`).
#[derive(Clone, Debug, Default)]
pub struct NodeBindParts {
    /// Per node, by `NodeId::index()`: entry offset in its block, or
    /// `NO_NODE_BIND` for the empty data.
    slots: Vec<u8>,
    /// Per block of `NODE_BIND_BLOCK` nodes: the index of its first entry.
    bases: Vec<u32>,
    entries: Vec<NodeBindData>,
}

const NODE_BIND_BLOCK_BITS: usize = 7;
const NODE_BIND_BLOCK: usize = 1 << NODE_BIND_BLOCK_BITS;
const NO_NODE_BIND: u8 = u8::MAX;

/// The empty binder data.
static EMPTY_NODE_BIND: NodeBindData = NodeBindData {
    symbol: SymbolId::NIL,
    local_symbol: SymbolId::NIL,
    locals: SymbolTable::NIL,
    next_container: Node::NIL,
    flow_node: FlowNodeId::NIL,
    end_flow_node: FlowNodeId::NIL,
    return_flow_node: FlowNodeId::NIL,
    added_flags: NodeFlags::NONE,
};

impl NodeBindParts {


    /// The distinct data entries, for remapping ids in place. Non-empty data
    /// stays non-empty and distinct under an id remap.
    pub fn entries_mut(&mut self) -> &mut [NodeBindData] {
        &mut self.entries
    }

    /// The stored form: the per-node slots, the block bases and the
    /// entries. The lib bind snapshot (`binder/lib_snapshot.rs`) dumps it.
    #[must_use]
    pub fn parts(&self) -> (&[u8], &[u32], &[NodeBindData]) {
        (&self.slots, &self.bases, &self.entries)
    }

}



/// Go `...any` diagnostic arguments. Go formats each with `%v`; we use
/// `ToString`. Example: `args![self.type_to_string(t), count]`.
#[macro_export]
macro_rules! args {
    ($($arg:expr),* $(,)?) => { vec![$(::std::string::ToString::to_string(&$arg)),*] };
}

/// Where a `LinkStore` keeps the record of a key.
#[derive(Clone, Copy, Debug)]
pub enum LinkSlot {
    /// An arena handle (`TypeId`, ...): an index into one page table for
    /// the whole arena.
    Arena(usize),
    /// A binder lineage index (`SymbolId`). The lineage never reuses an
    /// index, so in a long watch or editor session most indexes belong to
    /// dead file versions, which a new checker never reads. Indexes below
    /// `LINK_FLAT_IDS` use the arena table, as `Arena` does. The others use
    /// one page table per block of indexes that the store reads
    /// (`LinkStore::far_cell`), so a store pays only for the blocks it uses.
    Lineage(usize),
    /// A node or flow node handle: (file, local index + 1). Each file has
    /// its own page table.
    File(usize, usize),
    /// Any other key (a synthetic node): the hash map.
    Map,
}

/// A `LinkStore` key. Arena handles are dense small indexes, and node and
/// flow node handles are dense small indexes within a file, so their links
/// live in pages instead of a hash map.
pub trait LinkKey: Copy + Eq + std::hash::Hash {
    /// Where the record of this key lives.
    fn link_slot(self) -> LinkSlot;
}

macro_rules! arena_link_key {
    ($($name:ident),*) => {$(
        impl LinkKey for $name {
            #[inline]
            fn link_slot(self) -> LinkSlot {
                LinkSlot::Arena(self.index())
            }
        }
    )*};
}

arena_link_key!(
    TypeId,
    SignatureId,
    IndexInfoId,
    TypePredicateId,
    MapperId,
    InferenceContextId,
    SymbolTable
);

impl LinkKey for SymbolId {
    #[inline]
    fn link_slot(self) -> LinkSlot {
        LinkSlot::Lineage(self.index())
    }
}

/// Lineage indexes below this use the flat arena table: at most 512 KiB of
/// page pointers per store. A CLI build stays below it, so its reads are the
/// reads of an `Arena` key. (wgrowth1: a flat table for all indexes grew by
/// 8 bytes per 64 indexes ever bound, in each symbol link store of each
/// checker: about 7 MiB per config edit of a 600-file project.)
const LINK_FLAT_IDS: usize = 1 << 22;

/// Lineage indexes from `LINK_FLAT_IDS` on are in blocks of this many, with
/// one page table per block.
const LINK_BLOCK_SHIFT: usize = 16;

/// The block (an index into `LinkStore::files`) and the index in its block
/// of lineage index `index`. An index below `LINK_FLAT_IDS` wraps to a block
/// past every block table, so a lookup of it finds no page.
#[inline]
fn far_slot(index: usize) -> (usize, usize) {
    let far = index.wrapping_sub(LINK_FLAT_IDS);
    (far >> LINK_BLOCK_SHIFT, far & ((1 << LINK_BLOCK_SHIFT) - 1))
}

/// Files with an index below this get paged node slots. Synthetic nodes
/// (a file index near `u32::MAX`) use the hash map.
const LINK_MAX_FILES: u64 = 1 << 20;

/// The slot of a (file << 32 | local + 1) handle.
#[inline]
fn file_link_slot(handle: u64) -> LinkSlot {
    let file = handle >> 32;
    if file < LINK_MAX_FILES {
        LinkSlot::File(file as usize, (handle & 0xffff_ffff) as usize)
    } else {
        LinkSlot::Map
    }
}

impl LinkKey for Node {
    #[inline]
    fn link_slot(self) -> LinkSlot {
        file_link_slot(self.0)
    }
}

impl LinkKey for FlowNodeId {
    #[inline]
    fn link_slot(self) -> LinkSlot {
        file_link_slot(self.0)
    }
}

/// Keys per page. Pages are made on first use, so a store that few keys use
/// stays small.
const LINK_PAGE_SIZE: usize = 1 << 6;

/// The records of `LINK_PAGE_SIZE` keys, `None` for a key with no record.
type LinkPage<V> = [Option<V>; LINK_PAGE_SIZE];

/// A page table: for each page of `LINK_PAGE_SIZE` keys, the page, or `None`
/// when no key of the page has a record.
type PageTable<V> = Vec<Option<Box<LinkPage<V>>>>;

/// Makes the record of a key that has none.
// PERF: out of line, so an inlined `get` stays small; not `#[cold]`, because
// `get` calls it for every new record.
#[inline(never)]
fn new_link_record<V: Default>(cell: &mut Option<V>) -> &mut V {
    cell.insert(V::default())
}

/// Go `core.LinkStore[K, V]`: lazily created per-key link records.
/// Dense keys keep their records in pages (Go `core.PagedLinkStore`,
/// tsgo#4329). Other keys use a hash map. `get` reads an existing record
/// inline and adds a new page out of line.
///
/// A page holds its values in place, so a node key reads 4 dependent loads
/// (the list of file tables, the table of the file, the page, the value)
/// and an arena key 3. Each page costs 64 values, used or not, so a value must be 32
/// bytes or less (a page of 2 KiB at most). Put a larger value, or one that
/// few keys of a page have, in a `Box` (`LinkStore<Node, Box<V>>`, like Go
/// `symbolArenaLinkStore`): a page then costs 8 bytes per key, and a read
/// one more load.
#[derive(Clone, Debug)]
pub struct LinkStore<K: LinkKey, V: Default> {
    /// Page table of arena keys: one flat table for the whole arena (for
    /// lineage keys, up to `LINK_FLAT_IDS`).
    arena: PageTable<V>,
    /// Page tables of file keys, by file. A store of lineage keys has no
    /// file keys: it keeps the page tables of its blocks here (`far_slot`).
    files: Vec<PageTable<V>>,
    map: FxHashMap<K, V>,
}

impl<K: LinkKey, V: Default> Default for LinkStore<K, V> {
    fn default() -> Self {
        Self {
            arena: Vec::new(),
            files: Vec::new(),
            map: FxHashMap::default(),
        }
    }
}

impl<K: LinkKey, V: Default> LinkStore<K, V> {
    /// The record cell of a key with slots, or `None` when its page is
    /// absent.
    // PERF: a lineage key inside the flat table reads like an arena key.
    #[inline(always)]
    fn cell(&self, slot: LinkSlot) -> Option<&Option<V>> {
        let (table, index) = match slot {
            LinkSlot::Arena(index) => (&self.arena, index),
            LinkSlot::Lineage(index) if index / LINK_PAGE_SIZE < self.arena.len() => {
                (&self.arena, index)
            }
            LinkSlot::Lineage(index) => return self.far_cell(index),
            LinkSlot::File(file, index) => (self.files.get(file)?, index),
            LinkSlot::Map => return None,
        };
        let page = table.get(index / LINK_PAGE_SIZE)?.as_deref()?;
        Some(&page[index % LINK_PAGE_SIZE])
    }

    /// `cell` of a lineage key past the flat table: its block page, or
    /// `None` when the page is absent or the key belongs in the flat table.
    #[inline(never)]
    fn far_cell(&self, index: usize) -> Option<&Option<V>> {
        self.far_lookup(index)
    }

    #[inline(always)]
    fn far_lookup(&self, index: usize) -> Option<&Option<V>> {
        let (block, index) = far_slot(index);
        let page = self
            .files
            .get(block)?
            .get(index / LINK_PAGE_SIZE)?
            .as_deref()?;
        Some(&page[index % LINK_PAGE_SIZE])
    }






    #[cold]
    #[inline(never)]
    fn map_get(&mut self, key: K) -> &mut V {
        self.map.entry(key).or_default()
    }



    /// Go `store.Has(key)`.
    #[inline]
    #[must_use]
    pub fn has(&self, key: K) -> bool {
        match key.link_slot() {
            LinkSlot::Map => self.map.contains_key(&key),
            slot => self.cell(slot).is_some_and(Option::is_some),
        }
    }

    /// Go `store.TryGet(key)`.
    #[inline]
    #[must_use]
    pub fn try_get(&self, key: K) -> Option<&V> {
        match key.link_slot() {
            LinkSlot::Map => self.map.get(&key),
            slot => self.cell(slot)?.as_ref(),
        }
    }
}

// PORT: no Go counterpart. The filter of a symbol table (`Table::filter`)
// never hides an entry: every way to add, remove or rename entries keeps it.
#[cfg(test)]
mod table_filter_tests {
    use super::*;

    #[test]
    fn table_filter_keeps_every_entry_findable() {
        let mut arena = SymbolArena::new();
        let name = |n: usize| Name::from(format!("corefix1_name_{n}").as_str());
        for size in [0, 1, 7, 8, 9, 40, 255, 256, 300] {
            let set = arena.new_table();
            let built =
                arena.push_table_from_entries((0..size).map(|n| (name(n), SymbolId(n as u32 + 1))));
            for n in 0..size {
                arena.set(set, name(n), SymbolId(n as u32 + 1));
            }
            for n in (0..size).step_by(3) {
                arena.delete(set, name(n).as_str());
            }
            for n in 0..size + 50 {
                let expected = if n < size {
                    SymbolId(n as u32 + 1)
                } else {
                    SymbolId::NIL
                };
                assert_eq!(
                    arena.get_name(built, &name(n)),
                    expected,
                    "built {size} {n}"
                );
                assert_eq!(
                    arena.get(built, name(n).as_str()),
                    expected,
                    "built {size} {n}"
                );
                let expected = if n % 3 == 0 { SymbolId::NIL } else { expected };
                assert_eq!(arena.get_name(set, &name(n)), expected, "set {size} {n}");
                assert_eq!(arena.get(set, name(n).as_str()), expected, "set {size} {n}");
            }
            let copy = arena.clone_table(set);
            for n in 0..size {
                assert_eq!(
                    arena.get_name(copy, &name(n)),
                    arena.get_name(set, &name(n))
                );
            }
        }
        // A table made with room for 200 entries grows past its index, and
        // `reserve` grows it again.
        let grown = arena.new_table_with_capacity(200);
        for n in 0..300 {
            arena.set(grown, name(n), SymbolId(n as u32 + 1));
            if n == 280 {
                arena.reserve(grown, 1000);
            }
        }
        for n in 0..350 {
            let expected = if n < 300 {
                SymbolId(n as u32 + 1)
            } else {
                SymbolId::NIL
            };
            assert_eq!(arena.get_name(grown, &name(n)), expected, "grown {n}");
            assert_eq!(arena.get(grown, name(n).as_str()), expected, "grown {n}");
        }
    }
}

// PORT: no Go counterpart. The pages of `LinkStore` (AST node records step 7).
#[cfg(test)]
mod link_store_tests {
    use super::*;

    // `get` makes a default record, `insert_new` a record with a value, and
    // `has` and `try_get` see only keys with a record: not the other keys of
    // a page, a file or the map. File keys, synthetic (map) keys and arena
    // keys with a boxed value.
    #[test]
    fn link_store_pages_keep_records_by_key() {
        let node = |file: u64, local: u64| Node(file << 32 | local);
        let mut nodes = LinkStore::<Node, u32>::default();
        let a = node(3, 5);
        assert!(!nodes.has(a) && nodes.try_get(a).is_none());
        *nodes.get(a) = 7;
        assert_eq!(nodes.try_get(a), Some(&7));
        assert!(!nodes.has(node(3, 6)), "a key in the same page");
        assert_eq!(*nodes.get(node(3, 6)), 0, "get makes a default record");
        assert!(nodes.has(node(3, 6)));
        assert!(!nodes.has(node(1, 5)), "a key of another file");
        let far = node(0, 64 * 1000 + 1);
        *nodes.insert_new(far, 9) += 1;
        assert_eq!(nodes.try_get(far), Some(&10));
        let synthetic = node(u64::from(u32::MAX) - 1, 5);
        *nodes.get(synthetic) = 4;
        assert_eq!(nodes.try_get(synthetic), Some(&4));
        assert!(!nodes.has(node(u64::from(u32::MAX) - 1, 6)));

        let mut symbols = LinkStore::<SymbolId, Box<[u64; 8]>>::default();
        symbols.get(SymbolId(70))[1] = 2;
        symbols.insert_new(SymbolId(1), Box::new([1; 8]));
        assert_eq!(symbols.try_get(SymbolId(70)).map(|v| v[1]), Some(2));
        assert_eq!(symbols.try_get(SymbolId(1)).map(|v| v[7]), Some(1));
        assert!(!symbols.has(SymbolId(71)) && !symbols.has(SymbolId(64 * 50)));
    }

    // Symbol keys from `LINK_FLAT_IDS` on keep their records in pages by
    // block: a store with a few far keys has no page table entries for the
    // indexes between them, and every key still finds its own record.
    #[test]
    fn link_store_far_symbol_keys_use_blocks() {
        let flat = LINK_FLAT_IDS as u32;
        let block = 1u32 << LINK_BLOCK_SHIFT;
        let mut symbols = LinkStore::<SymbolId, u64>::default();
        let keys = [
            SymbolId(5),
            SymbolId(flat - 1),
            SymbolId(flat),
            SymbolId(flat + 70),
            SymbolId(flat + 900 * block + 3),
            SymbolId(u32::MAX - 1),
        ];
        for (n, key) in keys.into_iter().enumerate() {
            assert!(!symbols.has(key) && symbols.try_get(key).is_none());
            *symbols.get(key) = n as u64 + 1;
        }
        *symbols.insert_new(SymbolId(flat + 900 * block + 4), 40) += 2;
        for (n, key) in keys.into_iter().enumerate() {
            assert_eq!(symbols.try_get(key), Some(&(n as u64 + 1)));
            *symbols.get(key) += 10;
            assert_eq!(symbols.try_get(key), Some(&(n as u64 + 11)));
        }
        assert_eq!(symbols.try_get(SymbolId(flat + 900 * block + 4)), Some(&42));
        for absent in [
            6,
            flat - 2,
            flat + 1,
            flat + 64,
            flat + 899 * block,
            u32::MAX - 2,
        ] {
            assert!(!symbols.has(SymbolId(absent)), "{absent}");
        }
        assert_eq!(symbols.arena.len(), LINK_FLAT_IDS / LINK_PAGE_SIZE);
        let far_entries: usize = symbols.files.iter().map(Vec::len).sum();
        assert!(far_entries < 3 * (1 << LINK_BLOCK_SHIFT) / LINK_PAGE_SIZE);
    }
}

// Go panics and unported hits: `gopanic.rs` in goport_util.
pub use crate::goport_util::core::*;

/// Runs `f` and returns the text of the Go panic that it raises. Fails the
/// test when `f` does not panic or raises a plain Rust panic. Tests of the
/// Go `debug.*` sites (`gostd::debug`) use it.
#[cfg(test)]
pub(crate) fn go_panic_text(f: impl FnOnce()) -> String {
    let payload = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).expect_err("no panic");
    match payload.downcast::<GoPanic>() {
        Ok(p) => p.message,
        Err(_) => panic!("not a GoPanic"),
    }
}



static NEXT_PROGRAM_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// A new `GoProgram::id`: 1 for the first program of the process, then 2, 3...
#[must_use]
pub fn next_program_id() -> u32 {
    NEXT_PROGRAM_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}



/// Set by `register_program_version`. Then `set_prog` panics.
static MULTI: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// True in a multi-program process (watch, language server, tests): a
/// program version was registered, and `release_program` can free checkers.
#[inline]
pub fn is_multi_program() -> bool {
    MULTI.load(std::sync::atomic::Ordering::Relaxed)
}










// Go: core/version.go:8 version
// This is a var so it can be overridden by ldflags.
// PORT: Go release builds set it with `-ldflags -X ...core.version=<v>`
// (Herebyfile.mjs getReleaseBuildFlags). The port reads the build-time env
// var GOPORT_BUILD_VERSION instead (build-release.sh: RELEASE_VERSION).
// Without it the value is Go's default, as in the pinned reference build.

// Go: core/version.go:10 Version

// Go: core/version.go:14 versionMajorMinor
// PORT: a const, so a GOPORT_BUILD_VERSION with no second '.' stops the
// build where Go panics at start.

// Go: core/version.go:31 VersionMajorMinor
