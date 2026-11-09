use crate::prelude::*;
use crate::smallvec::SmallVec;

// PORT: Go `*InferenceState` is pooled on the checker
// (`Checker::freeinference_state: Option<Rc<RefCell<InferenceState>>>`).
// PERF: `infer_types` holds the one mutable borrow of the pooled state for
// the whole inference, and every inference function takes
// `n: &mut InferenceState`, so the hot paths make no `RefCell` checks. A
// nested inference takes another state from the pool, never this one.

// Go: checker/inference.go:11 InferenceKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InferenceKey {
    pub s: TypeId,
    pub t: TypeId,
}

// PORT: both ids are hashed as one word. No code iterates maps with these
// keys.

// PORT: perf. The same word through an Fx mix, which is the hash that
// `FxHashMap` computes for this key.
impl FlatKey for InferenceKey {
    #[inline]
    fn flat_hash(&self) -> u64 {
        use std::hash::Hasher;
        let mut h = crate::rustc_hash::FxHasher::default();
        h.write_u64(((self.s.0 as u64) << 32) | self.t.0 as u64);
        h.finish()
    }
}

/// The map type of `InferenceState::visited`. For an A/B run against
/// hashbrown, change it to `FxHashMap<InferenceKey, InferencePriority>`.
pub type InferenceVisitedMap = FlatMap<InferenceKey, InferencePriority>;

/// PORT: perf. Not in Go. `stack_recursion_id` as one word, so the inference
/// stacks store and compare ids with plain 8-byte moves. 0 is `None`. A node
/// handle has nonzero low 32 bits, so it is its own key. A symbol or type
/// id goes in the high 32 bits with zero low bits: a symbol as its id, a
/// type as its id with bit 31 set. An id that does not fit (a node with
/// zero low bits, a nil symbol, or a symbol or type id of 2^31 or more) is
/// stored as `None`. Such an entry is not a mapped type, an indexed access
/// or an intersection, so `has_matching_recursion_identity` on it only
/// compares `get_recursion_identity_from_target` and gives the same answer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecursionKey(u64);

impl RecursionKey {
    pub const NONE: Self = Self(0);
    const TYPE_BIT: u32 = 1 << 31;

    #[inline]
    #[must_use]
    pub fn new(id: Option<RecursionId>) -> Self {
        match id {
            Some(RecursionId::Node(n)) if n.0 as u32 != 0 => Self(n.0),
            Some(RecursionId::Symbol(s)) if s.0 != 0 && s.0 < Self::TYPE_BIT => {
                Self(u64::from(s.0) << 32)
            }
            Some(RecursionId::Type(t)) if t.0 < Self::TYPE_BIT => {
                Self(u64::from(t.0 | Self::TYPE_BIT) << 32)
            }
            _ => Self::NONE,
        }
    }
}

impl From<RecursionKey> for Option<RecursionId> {
    #[inline]
    fn from(key: RecursionKey) -> Self {
        let high = (key.0 >> 32) as u32;
        if key.0 as u32 != 0 {
            Some(RecursionId::Node(Node(key.0)))
        } else if high == 0 {
            None
        } else if high & RecursionKey::TYPE_BIT != 0 {
            Some(RecursionId::Type(TypeId(high & !RecursionKey::TYPE_BIT)))
        } else {
            Some(RecursionId::Symbol(SymbolId(high)))
        }
    }
}

impl FlatKey for RecursionKey {
    // The murmur3 finalizer: node keys differ in both halves and symbol and
    // type keys only in the high half, so both halves are mixed into the
    // low bits.
    #[inline]
    fn flat_hash(&self) -> u64 {
        let mut x = self.0;
        x ^= x >> 33;
        x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
        x ^ (x >> 33)
    }
}

/// `StackEntry::prev` of an entry with no earlier same-id entry.
const NO_PREV: u32 = u32::MAX;

/// PORT: perf. Not in Go. Per entry of an `InferenceStack`: `id` is
/// `RecursionKey::new(stack_recursion_id(t))`, `prev` is the index of the
/// next lower entry with the same id (or `NO_PREV`), and `steps` counts the
/// same-id entry pairs up to this entry whose type id does not go down.
// PERF: the three fields share one 16-byte element, so a push reads the id
// and the steps of `prev` from one cache line.
#[derive(Clone, Copy, Debug)]
struct StackEntry {
    id: RecursionKey,
    prev: u32,
    steps: u32,
}

/// The id list that `is_deeply_nested_type_with_ids` reads.
impl From<StackEntry> for Option<RecursionId> {
    #[inline]
    fn from(entry: StackEntry) -> Self {
        entry.id.into()
    }
}

/// PORT: perf. Not in Go. Go `sourceStack` or `targetStack` of
/// `InferenceState`, with the recursion id of each entry and the data that
/// gives `is_deeply_nested_type` for the top entry in O(1).
///
/// The Go scan counts the first entry with the probe's id, then each later
/// entry with that id whose type id is not lower than the one before it.
/// The count only grows. When the probe is the top entry, `1 + steps` of the
/// top entry is that count, and each push finds the entry before it through
/// `top`. `top[id]` is the index of the topmost entry with `id` whenever
/// `id` is on the stack. An index left from a popped entry fails the
/// `prev < idx && entries[prev].id == id` test, so keys of `top` are never
/// removed.
#[derive(Default)]
pub struct InferenceStack {
    /// The entry types, in their own list for the scan.
    pub types: Vec<TypeId>,
    entries: Vec<StackEntry>,
    top: FlatMap<RecursionKey, u32>,
    /// Entries whose id is `RecursionKey::NONE`.
    none: u32,
}

impl InferenceStack {
    // PERF: `inline(always)`, because `#[inline]` left push and pop out of
    // line.
    #[inline(always)]
    pub fn push(&mut self, t: TypeId, id: RecursionKey) {
        let idx = self.types.len() as u32;
        let mut entry = StackEntry {
            id,
            prev: NO_PREV,
            steps: 0,
        };
        if id == RecursionKey::NONE {
            self.none += 1;
        } else if let Some(prev) = self.top.insert(id, idx)
            && prev < idx
        {
            let prev_entry = self.entries[prev as usize];
            if prev_entry.id == id {
                entry.prev = prev;
                entry.steps = prev_entry.steps + u32::from(t >= self.types[prev as usize]);
            }
        }
        self.types.push(t);
        self.entries.push(entry);
    }

    #[inline(always)]
    pub fn pop(&mut self) {
        self.types.pop();
        let entry = self.entries.pop().unwrap();
        if entry.id == RecursionKey::NONE {
            self.none -= 1;
        } else if entry.prev != NO_PREV {
            self.top.insert(entry.id, entry.prev);
        }
    }

    /// The Go deeply nested answer for the top entry as the probe, when it
    /// needs no scan: the top has an id and no entry lacks one.
    #[inline]
    #[must_use]
    pub fn top_deeply_nested(&self, max_depth: i32) -> Option<bool> {
        let top = self.entries.last()?;
        if top.id == RecursionKey::NONE || self.none != 0 {
            return None;
        }
        let steps = top.steps;
        let max_depth = i64::from(max_depth);
        Some(self.types.len() as i64 >= max_depth && 1 + i64::from(steps) >= max_depth)
    }

    /// Empties the stack and keeps the allocations. `top` keeps its stale
    /// indexes (they fail the validity test), but a large `top` is dropped
    /// so ids of earlier root inferences do not fill it.
    pub fn clear(&mut self) {
        self.types.clear();
        self.entries.clear();
        self.none = 0;
        if self.top.len() > 1024 {
            self.top = FlatMap::default();
        }
    }
}

// Go: checker/inference.go:16 InferenceState
// PORT: Go `inferences []*InferenceInfo` is the inference list of an inference
// context, so it is the `InferenceContextId` that owns the list (nil when the
// state is in the free pool). Callers that pass a fresh list in Go push a
// scratch context first. Go nil `visited` map is `None`.
#[derive(Default)]
pub struct InferenceState {
    pub inferences: InferenceContextId,
    pub original_source: TypeId,
    pub original_target: TypeId,
    pub priority: InferencePriority,
    pub inference_priority: InferencePriority,
    pub contravariant: bool,
    pub bivariant: bool,
    pub expanding_flags: ExpandingFlags,
    pub propagation_type: TypeId,
    pub visited: Option<InferenceVisitedMap>,
    // PORT: perf. Not in Go. The largest `visited` length that a root
    // inference on this pooled state reached (see `invoke_once`).
    pub max_visited_len: usize,
    // PORT: perf. Go `sourceStack` and `targetStack` with the recursion id
    // of each entry (see `InferenceStack`).
    pub source_stack: InferenceStack,
    pub target_stack: InferenceStack,
    pub next: Option<Rc<RefCell<InferenceState>>>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursion_key_round_trip() {
        let ids = [
            RecursionId::Node(Node(1)),
            RecursionId::Node(Node((7 << 32) | 3)),
            RecursionId::Node(Node((0xffff_fffe << 32) | 0xffff_ffff)),
            RecursionId::Symbol(SymbolId(1)),
            RecursionId::Symbol(SymbolId(0x7fff_ffff)),
            RecursionId::Type(TypeId(0)),
            RecursionId::Type(TypeId(1)),
            RecursionId::Type(TypeId(0x7fff_ffff)),
        ];
        for (i, &a) in ids.iter().enumerate() {
            let key = RecursionKey::new(Some(a));
            assert_ne!(key, RecursionKey::NONE);
            assert_eq!(Option::<RecursionId>::from(key), Some(a));
            for &b in &ids[i + 1..] {
                assert_ne!(key, RecursionKey::new(Some(b)));
            }
        }
        for id in [
            None,
            Some(RecursionId::Node(Node(5 << 32))),
            Some(RecursionId::Symbol(SymbolId(0))),
            Some(RecursionId::Symbol(SymbolId(0x8000_0000))),
            Some(RecursionId::Type(TypeId(0x8000_0000))),
        ] {
            assert_eq!(RecursionKey::new(id), RecursionKey::NONE);
        }
    }

    /// The Go scan of `isDeeplyNestedType` with the top entry as the probe,
    /// comparing stored ids.
    fn scan(stack: &InferenceStack, max_depth: i32) -> bool {
        if (stack.types.len() as i32) < max_depth {
            return false;
        }
        let probe = stack.entries.last().unwrap().id;
        let mut count = 0;
        let mut last = TypeId(0);
        for (&t, entry) in stack.types.iter().zip(&stack.entries) {
            if entry.id == probe {
                if t >= last {
                    count += 1;
                    if count >= max_depth {
                        return true;
                    }
                }
                last = t;
            }
        }
        false
    }

    #[test]
    fn stack_answer_matches_scan() {
        let keys = [
            RecursionKey::NONE,
            RecursionKey::new(Some(RecursionId::Node(Node((1 << 32) | 9)))),
            RecursionKey::new(Some(RecursionId::Symbol(SymbolId(9)))),
            RecursionKey::new(Some(RecursionId::Type(TypeId(9)))),
            RecursionKey::new(Some(RecursionId::Type(TypeId(10)))),
        ];
        let mut stack = InferenceStack::default();
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        let mut fast_answers = 0;
        for _ in 0..200_000 {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            let r = rng >> 16;
            match r % 100 {
                0 => stack.clear(),
                1..=54 if stack.types.len() < 30 => {
                    // NONE entries are rare, so most stacks have none.
                    let key = if r % 1000 < 20 {
                        keys[0]
                    } else {
                        keys[1 + (r / 100 % 4) as usize]
                    };
                    stack.push(TypeId((r / 1000 % 6) as u32 + 1), key);
                    let has_none = stack.entries.iter().any(|e| e.id == RecursionKey::NONE);
                    for max_depth in 1..=4 {
                        let fast = stack.top_deeply_nested(max_depth);
                        if has_none {
                            assert_eq!(fast, None);
                        } else {
                            assert_eq!(fast, Some(scan(&stack, max_depth)));
                            fast_answers += 1;
                        }
                    }
                }
                _ if !stack.types.is_empty() => stack.pop(),
                _ => {}
            }
        }
        assert!(fast_answers > 1000);
    }
}
