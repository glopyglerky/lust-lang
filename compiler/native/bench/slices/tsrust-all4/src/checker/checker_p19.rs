//! Go: checker/checker.go:17116-18041 (checkDeclarationInitializer ..
//! getTypeForBindingElementParent), including the cache key builder
//! (`CacheHashKey`, `keyBuilder` and the `get*Key` functions).

use crate::prelude::*;

use std::hash::Hasher as _;


// Go: checker/checker.go:17697 CacheHashKey
// PORT: Go `CacheHashKey` is an `xxh3.Uint128` (`{Hi, Lo uint64}`). It is
// 4-byte aligned so cache entries with a 4-byte value take 20 bytes, not 24.

// PORT: the key is already an xxh3 hash, so map hashing feeds only its low
// half to the hasher. Equality still compares both halves. No code depends on
// the iteration order of maps with these keys.


/// xxh3 default secret (`zeebo/xxh3` `key`, the XXH3 kSecret).
const XXH3_SECRET: [u8; 192] = [
    0xb8, 0xfe, 0x6c, 0x39, 0x23, 0xa4, 0x4b, 0xbe, 0x7c, 0x01, 0x81, 0x2c, 0xf7, 0x21, 0xad, 0x1c,
    0xde, 0xd4, 0x6d, 0xe9, 0x83, 0x90, 0x97, 0xdb, 0x72, 0x40, 0xa4, 0xa4, 0xb7, 0xb3, 0x67, 0x1f,
    0xcb, 0x79, 0xe6, 0x4e, 0xcc, 0xc0, 0xe5, 0x78, 0x82, 0x5a, 0xd0, 0x7d, 0xcc, 0xff, 0x72, 0x21,
    0xb8, 0x08, 0x46, 0x74, 0xf7, 0x43, 0x24, 0x8e, 0xe0, 0x35, 0x90, 0xe6, 0x81, 0x3a, 0x26, 0x4c,
    0x3c, 0x28, 0x52, 0xbb, 0x91, 0xc3, 0x00, 0xcb, 0x88, 0xd0, 0x65, 0x8b, 0x1b, 0x53, 0x2e, 0xa3,
    0x71, 0x64, 0x48, 0x97, 0xa2, 0x0d, 0xf9, 0x4e, 0x38, 0x19, 0xef, 0x46, 0xa9, 0xde, 0xac, 0xd8,
    0xa8, 0xfa, 0x76, 0x3f, 0xe3, 0x9c, 0x34, 0x3f, 0xf9, 0xdc, 0xbb, 0xc7, 0xc7, 0x0b, 0x4f, 0x1d,
    0x8a, 0x51, 0xe0, 0x4b, 0xcd, 0xb4, 0x59, 0x31, 0xc8, 0x9f, 0x7e, 0xc9, 0xd9, 0x78, 0x73, 0x64,
    0xea, 0xc5, 0xac, 0x83, 0x34, 0xd3, 0xeb, 0xc3, 0xc5, 0x81, 0xa0, 0xff, 0xfa, 0x13, 0x63, 0xeb,
    0x17, 0x0d, 0xdd, 0x51, 0xb7, 0xf0, 0xda, 0x49, 0xd3, 0x16, 0x55, 0x26, 0x29, 0xd4, 0x68, 0x9e,
    0x2b, 0x16, 0xbe, 0x58, 0x7d, 0x47, 0xa1, 0xfc, 0x8f, 0xf8, 0xb8, 0xd1, 0x7a, 0xd0, 0x31, 0xce,
    0x45, 0xcb, 0x3a, 0x8f, 0x95, 0x16, 0x04, 0x28, 0xaf, 0xd7, 0xfb, 0xca, 0xbb, 0x4b, 0x40, 0x7e,
];

const XXH_PRIME32_1: u64 = 2654435761;
const XXH_PRIME32_2: u64 = 2246822519;
const XXH_PRIME32_3: u64 = 3266489917;
const XXH_PRIME64_1: u64 = 11400714785074694791;
const XXH_PRIME64_2: u64 = 14029467366897019727;
const XXH_PRIME64_3: u64 = 1609587929392839161;
const XXH_PRIME64_4: u64 = 9650029242287828579;
const XXH_PRIME64_5: u64 = 2870177450012600261;
const XXH_STRIPE: usize = 64;
const XXH_BLOCK: usize = 1024;




#[inline(always)]
fn xxh_mul_fold64(x: u64, y: u64) -> u64 {
    let p = (x as u128).wrapping_mul(y as u128);
    (p >> 64) as u64 ^ p as u64
}

#[inline(always)]
fn xxh3_avalanche(mut x: u64) -> u64 {
    x ^= x >> 37;
    x = x.wrapping_mul(0x165667919e3779f9);
    x ^ (x >> 32)
}

#[inline(always)]
fn xxh64_avalanche_small(mut x: u64) -> u64 {
    x = x.wrapping_mul(XXH_PRIME64_2);
    x ^= x >> 29;
    x = x.wrapping_mul(XXH_PRIME64_3);
    x ^ (x >> 32)
}









// Go: checker/checker.go:17703 keyBuilder
// PORT: Go hashes with `xxh3.Hash128`. The crate has no xxh3 dependency and
// the contract forbids new ones, so `xxh3_hash128` in this file is a hand
// port of `zeebo/xxh3` `Hash128` (seed 0). A Go nil `overflowBuffer` is an
// empty one.
#[derive(Clone)]
pub struct KeyBuilder {
    pub inline_length: usize,
    pub overflow_buffer: Vec<u8>,
    pub inline_buffer: [u8; KEY_BUILDER_INLINE_BUFFER_LEN],
}

/// Go `len(keyBuilder.inlineBuffer)`.
const KEY_BUILDER_INLINE_BUFFER_LEN: usize = 192;

impl Default for KeyBuilder {
    #[inline]
    fn default() -> Self {
        KeyBuilder {
            inline_length: 0,
            overflow_buffer: Vec::new(),
            inline_buffer: [0; KEY_BUILDER_INLINE_BUFFER_LEN],
        }
    }
}


impl KeyBuilder {
    // Go: checker/checker.go:17709 keyBuilder.hash

    // Go: checker/checker.go:17718 keyBuilder.spill
    // spill moves the buffered bytes onto the end of overflowBuffer, so the key's byte
    // stream stays overflowBuffer followed by inlineBuffer.
    #[cold]
    #[inline(never)]
    fn spill(&mut self) {
        self.overflow_buffer
            .extend_from_slice(&self.inline_buffer[..self.inline_length]);
        self.inline_length = 0;
    }

    // Go: checker/checker.go:17723 keyBuilder.writeByte
    #[inline]
    pub fn write_byte(&mut self, c: u8) {
        if self.inline_length == self.inline_buffer.len() {
            self.spill();
        }
        self.inline_buffer[self.inline_length] = c;
        self.inline_length += 1;
    }

    // Go: checker/checker.go:17731 keyBuilder.writeString

    // Go: checker/checker.go:17742 keyBuilder.writeUint32

    // Go: checker/checker.go:17750 keyBuilder.writeUint64

    // Go: checker/checker.go:17758 keyBuilder.writeInt

    // Go: checker/checker.go:17762 keyBuilder.writeSymbol
    // PORT: `ast.GetSymbolId` takes the symbol arena per the contract.

    // Go: checker/checker.go:17766 keyBuilder.writeType
    // PORT: Go writes `t.id`; the handle value is the type id.

    // Go: checker/checker.go:17770 keyBuilder.writeTypes
    // PERF: when the whole list fits in the inline buffer, it is written with
    // one room test. The bytes are the same as `write_int(len)` and then
    // `write_type` for each type.

    // Go: checker/checker.go:17777 keyBuilder.writeAlias

    // Go: checker/checker.go:17787 keyBuilder.writeGenericTypeReferences
    // PORT: Go reaches the checker through `t.checker`; it is passed in.

    // PORT: the recursive `writeTypeReference` closure inside Go
    // `keyBuilder.writeGenericTypeReferences`, with its captured variables
    // passed explicitly.

    // Go: checker/checker.go:17822 keyBuilder.writeNodeId
    // PORT: Go `ast.NodeId` is a `uint64` (`get_node_id` returns `u64`).

    // Go: checker/checker.go:17826 keyBuilder.writeNode
}

/// Size of the `ShortKey` buffer. A type list and an alias with 4 types each
/// and a flag byte take 58 bytes.
const SHORT_KEY_CAP: usize = 64;

/// Byte length of Go `keyBuilder.writeTypes` for `n` types: the count as 8
/// bytes and then 4 bytes for each type.
#[inline(always)]
const fn type_list_key_len(n: usize) -> usize {
    8 + 4 * n
}

/// Byte length of Go `keyBuilder.writeAlias`: 1 for a nil alias, else the
/// tag byte, the 8-byte symbol id and the type list.
#[inline(always)]
fn alias_key_len(alias: Option<(SymbolId, &[TypeId])>) -> usize {
    match alias {
        Some((_, types)) => 9 + type_list_key_len(types.len()),
        None => 1,
    }
}

/// PERF: a cache key built in a 64-byte stack array, for the hot
/// `getTypeInstantiationKey`, `getConditionalTypeKey` and
/// `getIntersectionKey`. A `KeyBuilder` zero-fills 192 bytes and owns an
/// overflow vector, and its type list copy loop becomes a `memcpy` call.
/// Here the fill is 64 bytes (safe code must initialize the array; this is
/// a few vector stores and no call), and lists of up to 4 types are
/// fixed-size stores. The caller tests the room for the whole key once and
/// uses the `KeyBuilder` path for a longer key, so the bounds checks of the
/// stores never fail. The bytes and their order are the `KeyBuilder` bytes,
/// so the xxh3 hash is the same.
struct ShortKey {
    len: usize,
    buf: [u8; SHORT_KEY_CAP],
}

impl ShortKey {
    #[inline(always)]
    fn new() -> Self {
        ShortKey {
            len: 0,
            buf: [0; SHORT_KEY_CAP],
        }
    }




}

// PORT: the Go `get*Key` functions below are package functions. The ones
// that only use type ids (and alias symbols) are free functions; functions
// that write aliases take the symbol arena because `ast.GetSymbolId` does.
// Each also has a `Checker` method with the same name and Go parameters
// (supplying `&self.symbols`), so callers can use either form.

// Go: checker/checker.go:17832 getTypeListKey
// PERF: a list of 1 or 2 types is a 12 or 16 byte key (the count as 8 bytes,
// then 4 bytes for each type). It is built in a stack array and hashed with
// no `KeyBuilder`. Same bytes, so the same key.


// Go: checker/checker.go:17838 getAliasKey

// Go: checker/checker.go:17868 getIntersectionKey
// PERF: built in a `ShortKey` when the whole key fits in it, else through
// `keyBuilder`. Same bytes, so the same key.


// Go: checker/checker.go:17879 getTupleKey

// Go: checker/checker.go:17902 getTypeAliasInstantiationKey

// Go: checker/checker.go:17906 getTypeInstantiationKey



// Go: checker/checker.go:17916 getIndexedAccessKey

// Go: checker/checker.go:17925 getTemplateTypeKey

// Go: checker/checker.go:17939 getConditionalTypeKey

// Go: checker/checker.go:17967 getNodeListKey


#[cfg(test)]
mod key_hash_tests {
    use super::*;

    /// `xxh3_hash128` matches the xxhash-rust `xxh3_128` for every length
    /// path, and `type_key_no_alias` matches the `KeyBuilder` key.
    #[test]
    fn xxh3_and_type_key_match_reference() {
        let mut x = 0x2545_f491_4f6c_dd1du64;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        let bytes: Vec<u8> = (0..4096).map(|_| next() as u8).collect();
        for l in 0..bytes.len() {
            let p = &bytes[..l];
            let want = crate::xxhash_rust::xxh3::xxh3_128(p);
            assert_eq!(
                xxh3_hash128(p),
                ((want >> 64) as u64, want as u64),
                "len {l}"
            );
        }
        for _ in 0..100_000 {
            let t = TypeId(next() as u32);
            let mut b = KeyBuilder::default();
            b.write_type(t);
            // `write_alias(None)` writes this byte.
            b.write_byte(0);
            assert_eq!(type_key_no_alias(t), b.hash());
        }
    }

    /// `KeyBuilder` hashes the bytes it was given, in order, for keys that
    /// stay inline and keys that spill past the inline buffer. The short
    /// `get_type_list_key` paths hash the same bytes as the builder.
    /// The test keeps its accepted name from before tsgo#4784 renamed
    /// `KeyHasher` to `KeyBuilder`.
    #[test]
    fn key_hasher_and_short_keys_match_bytes() {
        let mut x = 0x9e37_79b9_7f4a_7c15u64;
        let mut next = move || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x
        };
        for _ in 0..2_000 {
            let mut b = KeyBuilder::default();
            let mut want = Vec::new();
            let writes = next() % 200;
            for _ in 0..writes {
                let v = next();
                match v % 5 {
                    0 => {
                        b.write_byte(v as u8);
                        want.push(v as u8);
                    }
                    1 => {
                        b.write_uint32(v as u32);
                        want.extend((v as u32).to_le_bytes());
                    }
                    2 => {
                        b.write_uint64(v);
                        want.extend(v.to_le_bytes());
                    }
                    3 => {
                        // Short strings, and now and then one longer than the
                        // inline buffer (Go writes it straight to the overflow).
                        let n = if (v >> 52) % 16 == 0 {
                            200 + (v >> 40) as usize % 100
                        } else {
                            (v >> 60) as usize % 9
                        };
                        let s: String = (0..n)
                            .map(|i| (b'a' + ((v >> (i % 60)) % 26) as u8) as char)
                            .collect();
                        b.write_string(&s);
                        want.extend_from_slice(s.as_bytes());
                    }
                    _ => {
                        let types: Vec<TypeId> =
                            (0..(v >> 56) % 80).map(|_| TypeId(next() as u32)).collect();
                        b.write_types(&types);
                        want.extend((types.len() as u64).to_le_bytes());
                        for t in &types {
                            want.extend(t.0.to_le_bytes());
                        }
                    }
                }
            }
            let (hi, lo) = xxh3_hash128(&want);
            assert_eq!(b.hash(), CacheHashKey { hi, lo }, "len {}", want.len());
        }
        for _ in 0..100_000 {
            let (t0, t1) = (TypeId(next() as u32), TypeId(next() as u32));
            assert_eq!(get_type_list_key(&[t0]), type_list_key_with_builder(&[t0]));
            assert_eq!(
                get_type_list_key(&[t0, t1]),
                type_list_key_with_builder(&[t0, t1])
            );
        }
    }
}
