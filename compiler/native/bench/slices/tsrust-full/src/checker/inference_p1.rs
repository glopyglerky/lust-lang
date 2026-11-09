use crate::prelude::*;

// PORT: Go `*InferenceState` is pooled on the checker
// (`Checker::freeinference_state: Option<Rc<RefCell<InferenceState>>>`).
// PERF: `infer_types` holds the one mutable borrow of the pooled state for
// the whole inference, and every inference function takes
// `n: &mut InferenceState`, so the hot paths make no `RefCell` checks. A
// nested inference takes another state from the pool, never this one.

// Go: checker/inference.go:11 InferenceKey

// PORT: both ids are hashed as one word. No code iterates maps with these
// keys.

// PORT: perf. The same word through an Fx mix, which is the hash that
// `FxHashMap` computes for this key.


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




// Go: checker/inference.go:16 InferenceState
// PORT: Go `inferences []*InferenceInfo` is the inference list of an inference
// context, so it is the `InferenceContextId` that owns the list (nil when the
// state is in the free pool). Callers that pass a fresh list in Go push a
// scratch context first. Go nil `visited` map is `None`.


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
