//! Port of Go `checker/relater.go` lines 1-915: relation types, the public
//! relation entry points (`isTypeRelatedTo`, `checkTypeRelatedToEx`, ...),
//! simple and enum relations, error elaboration, weak type and known
//! property checks, recursion identities, and best matching type selection.

use crate::prelude::*;

// PORT: the Go flag types `SignatureCheckMode`, `MinArgumentCountFlags`,
// `IntersectionState`, `RecursionFlags`, `ExpandingFlags` and
// `RelationComparisonResult` (relater.go:18-75) live in `crate::flags`.
// PORT: Go `*Relation` is `Rc<RefCell<Relation>>`; a `*Relation` param is
// `&Rc<RefCell<Relation>>`. Go pointer equality on relations is `Rc::ptr_eq`.
// PORT: Go `diagnosticOutput *[]*ast.Diagnostic` is `Option<&mut Vec<Diagnostic>>`.
// PORT: a nullable Go `*diagnostics.Message` is `Option<&'static Message>`.

// Go: checker/relater.go:77 DiagnosticAndArguments

// Go: checker/relater.go:82 ErrorOutputContainer

// Go: checker/relater.go:87 ErrorReporter
// PORT: Go `ErrorReporter func(message *diagnostics.Message, args ...any)`.
// A nil-able `ErrorReporter` param is `Option<ErrorReporter<'_>>`. The
// callback gets the checker as its first argument (Go closures capture it).
// Use `reborrow_error_reporter` to pass an optional reporter on more than once.

// PORT: helper with no Go equivalent. Go passes the same `ErrorReporter`
// value to several callees; in Rust the `Option<&mut dyn FnMut>` must be
// reborrowed for each call.

// Go: checker/relater.go:89 RecursionId
// PORT: Go `RecursionId{value any}` holds one of `*ast.Node`, `*ast.Symbol`
// or `*Type`. Go interface equality compares the dynamic type and pointer,
// which this enum reproduces.





// Go: checker/relater.go:94 asRecursionId
// This function exists to constrain the types of values that can be used as recursion IDs.

/// Hasher for maps keyed by `CacheHashKey`. The key is already an xxh3 hash
/// and `CacheHashKey` hashes only its low half, so that half is the map hash
/// as is. Iteration order differs from `FxHashMap`, so use it only for maps
/// that no code iterates.
#[derive(Clone, Copy, Default)]
pub struct CacheKeyHasher(u64);

impl std::hash::Hasher for CacheKeyHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }

    // Only reached if a key other than `CacheHashKey` is used.
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ u64::from(b)).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }
}


// PORT: perf. `CacheHashKey` is already an xxh3 hash, so `FlatMap` uses its
// low half as is (the same bits `CacheKeyHasher` uses).

// PORT: perf. Not in Go. Go keys every relation result by the xxh3 hash of
// the key bytes. A plain key has only 12 bytes of content (Go writes `'s'`,
// the source and target ids and the intersection state), so the port keeps
// those values as the key: it needs no xxh3 and compares exactly. A generic
// key (`'g'`, with type references) stays Go's xxh3 hash. Go's xxh3 keys
// collide with a chance of about 2^-128; plain keys here never do.

/// The content of a plain relation key (Go `getRelationKey` writes `'s'`,
/// then these three values).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlainRelationKey {
    pub source: u32,
    pub target: u32,
    pub intersection_state: u32,
}

/// Bits of the source and of the target id in a packed plain key.
const PACKED_ID_BITS: u32 = 28;

impl PlainRelationKey {
}

impl FlatKey for PlainRelationKey {
    // A 64x64 to 128-bit multiply folded to 64 bits puts every key bit into
    // the low bits that pick the slot.
    #[inline]
    fn flat_hash(&self) -> u64 {
        let ids = u64::from(self.source) | u64::from(self.target) << 32;
        fold_mul(ids, u64::from(self.intersection_state))
    }
}

/// A 64x64 to 128-bit multiply of `x` and a constant changed by `y`, folded
/// to 64 bits, so every bit of `x` reaches the low bits.
#[inline]
fn fold_mul(x: u64, y: u64) -> u64 {
    let p = u128::from(x ^ 0x243f_6a88_85a3_08d3) * u128::from(0x9e37_79b9_7f4a_7c15 ^ y);
    (p >> 64) as u64 ^ p as u64
}

// Only for `RelationKeySet`: `CacheKeyHasher` takes one `u64` as the hash.


/// Bits of the result in a `PlainResultTable` entry.
const RESULT_BITS: u32 = 6;

// PORT: perf. Not in Go. The plain results of one relation, each in one
// `u64`: the packed key (`PlainRelationKey::packed`) in the high 58 bits and
// the result in the low 6 bits, so a cache line holds 8 results (a table of
// 12-byte keys and 4-byte results holds 4). Open addressing with linear
// probing, a power-of-two slot count and a load of at most 1/2; 0 is an
// empty slot. There is no iteration, so the layout cannot change any output.


// Go: checker/relater.go:98 Relation
// PORT: Go has one map of xxh3 keys. Here plain keys are in `plain` (or in
// `plain_wide` when they do not pack) and generic keys in `generic` (see
// `RelationKey`). There is no iteration, so this cannot change any output.


// PORT: perf. Not in Go. Go tests a relation by pointer (`relation ==
// c.identityRelation`). Hot code here tests a `RelationKind`, so it does not
// clone the `Rc` of the relation to keep it while `self` is borrowed.
/// One of the checker's five relations (see `Checker::relation_of`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RelationKind {
    #[default]
    Identity,
    Subtype,
    StrictSubtype,
    Assignable,
    Comparable,
}



// Go: checker/relater.go:400 createDiagnosticChainFromErrorChain
// PORT: Go `*ErrorChain` is read through `Option<&ErrorChain>`; callers pass
// `relater.error_chain.as_deref()`. A nil result is `None`.


// Go: checker/relater.go:743 isHyphenatedJsxName

#[cfg(test)]
mod tests {
    use super::*;

    /// `Relation` against a `HashMap`, with plain keys that pack, plain keys
    /// that do not, and generic keys, so all three tables grow and overwrite.
    #[test]
    fn relation_matches_hash_map() {
        let mut relation = Relation::default();
        let mut want = std::collections::HashMap::new();
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut next = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let results = [
            RelationComparisonResult::SUCCEEDED,
            RelationComparisonResult::FAILED,
            RelationComparisonResult::FAILED | RelationComparisonResult::COMPLEXITY_OVERFLOW,
            RelationComparisonResult::SUCCEEDED | RelationComparisonResult::REPORTS_MASK,
        ];
        for n in 0..20_000u64 {
            let v = next();
            let key = match v % 8 {
                0 => RelationKey::Generic(CacheHashKey {
                    hi: v >> 8 & 0xff,
                    lo: v >> 16 & 0x3ff,
                }),
                1 => RelationKey::Plain(PlainRelationKey {
                    source: (1 << PACKED_ID_BITS) + (v >> 8 & 0x3f) as u32,
                    target: (v >> 16 & 0x3f) as u32,
                    intersection_state: 0,
                }),
                _ => RelationKey::Plain(PlainRelationKey {
                    source: (v >> 8 & 0x3f) as u32,
                    target: (v >> 16 & 0x3f) as u32 | if v % 16 == 2 { 0xfff_ffc0 } else { 0 },
                    intersection_state: (v >> 24 & 3) as u32,
                }),
            };
            if n % 3 == 0 {
                let result = results[(v >> 32) as usize % results.len()];
                relation.set(key, result);
                want.insert(key, result);
                assert_eq!(relation.size() as usize, want.len());
            }
            let got = relation.get(key);
            assert_eq!(got, want.get(&key).copied().unwrap_or_default());
        }
    }
}
