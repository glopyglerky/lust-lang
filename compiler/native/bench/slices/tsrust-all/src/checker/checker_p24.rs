//! Port of `checker/checker.go` lines 21264-22202: mixins, union and
//! intersection properties, apparent and reduced types, type arguments and
//! defaults, named members, and the core of type instantiation.

use crate::checker::utilities_p1::{NO_SORT_ORDER, SymbolSortKey};
use crate::prelude::*;
use crate::smallvec::SmallVec;

/// Reusable buffers of `get_named_members` (the `Checker` field
/// `named_members_scratch`), so a call allocates only its result.
#[derive(Default)]
pub(crate) struct NamedMembersScratch {
    /// Candidate members, then the sorted result.
    symbols: Vec<SymbolId>,
    /// Sort keys: contained members first when the container is a class or
    /// interface, else all members.
    keys: Vec<SymbolSortKey>,
    /// Sort keys of the members that are not contained.
    other_keys: Vec<SymbolSortKey>,
    /// Ranges of the container's declarations, read once per call.
    container_ranges: Vec<TextRange>,
}





/// Go `core.AppendIfUnique` for the declarations list of
/// `create_union_or_intersection_property`, which can grow long. While the
/// list is short, an add scans it. From `SCAN` nodes on, an open-addressing
/// table of the listed nodes answers instead. A node is pushed only when it
/// is new, so the list keeps Go's order.
// Go: core/core.go:380 AppendIfUnique
#[derive(Default)]
struct DeclarationSet {
    /// Empty until the list reaches `SCAN` nodes, then a power-of-two table
    /// at most half full. `Node::NIL` marks a free slot.
    slots: Vec<Node>,
}

impl DeclarationSet {
    /// The list length from which the table answers. Lists are mostly
    /// shorter than 8 or longer than 32; blueprint runs fastest from 8 to 16.
    const SCAN: usize = 16;


    /// Makes a table of `len` slots (a power of two) that has each listed
    /// node.
    fn fill(&mut self, list: &[Node], len: usize) {
        self.slots = vec![Node::NIL; len];
        for &listed in list {
            self.insert(listed);
        }
    }

    /// Puts `node` in the table. False when the table has it.
    fn insert(&mut self, node: Node) -> bool {
        let mask = self.slots.len() - 1;
        // Fibonacci hashing: the top bits of the product pick the first slot.
        let shift = 64 - self.slots.len().trailing_zeros();
        let mut i = (node.0.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> shift) as usize;
        loop {
            let slot = self.slots[i];
            if slot.is_nil() {
                self.slots[i] = node;
                return true;
            }
            if slot == node {
                return false;
            }
            i = (i + 1) & mask;
        }
    }
}

/// Go `orderedSet.Add` for a small symbol set kept as a list. Lookups scan the
/// list while it is short; from `ORDERED_SYMBOL_SET_SCAN` symbols on, `index`
/// holds every member and answers them instead.
fn ordered_symbol_set_add(
    list: &mut SmallVec<[SymbolId; 8]>,
    index: &mut FxHashSet<SymbolId>,
    s: SymbolId,
) {
    const ORDERED_SYMBOL_SET_SCAN: usize = 32;
    if list.len() < ORDERED_SYMBOL_SET_SCAN {
        if !list.contains(&s) {
            list.push(s);
            if list.len() == ORDERED_SYMBOL_SET_SCAN {
                index.extend(list.iter().copied());
            }
        }
    } else if index.insert(s) {
        list.push(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker::utilities_p1::union_sort_tests::with_alias_types;

    /// The example of `some_property_reduces_to_never`. In
    /// `A<string> & T & { kind: "b" }`, where
    /// `T extends A<string> | A<number>`, the first `m` is the public method
    /// of `A<string>`. The `m` of `T` comes next: the union property of
    /// `A<string> | A<number>`, a property with the same value declaration
    /// (Go `createUnionOrIntersectionProperty`, checker.go:21821 and
    /// :21988). So it is skipped, and `m` stays with the public methods:
    /// `kind` (`"a" & "a" & "b"` is never, a discriminant, Go
    /// `isDiscriminantWithNeverType`, checker.go:22273) is tested first,
    /// though `m` is seen first, and the test returns there. The property
    /// cache of the intersection then has no `m`. With no skip, `m` would
    /// be tested first. Go keeps the order of an intersection
    /// (`addTypeToIntersection`, checker.go:26723), and distributes an
    /// intersection over a union (checker.go:26508), so
    /// `A<string> & (A<string> | B)` is a union.
    #[test]
    fn a_skipped_property_can_be_a_union_property() {
        const SOURCE: &str = r#"
interface A<X> { m(): X; kind: "a" }
interface B { m: number }
type D = A<string> & (A<string> | B);
type I<T extends A<string> | A<number>> = A<string> & T & { kind: "b" };
"#;
        let got = with_alias_types(SOURCE, |c, types| {
            let (d, i) = (types[0], types[1]);
            let constituents = c.ty(i).types().to_vec();
            let props: Vec<SymbolId> = constituents[..2]
                .iter()
                .map(|&u| {
                    let props = c.get_properties_of_type(u);
                    props
                        .iter()
                        .copied()
                        .find(|&p| c.sym(p).name == "m")
                        .expect("A<string> and T have m")
                })
                .collect();
            let public_methods: Vec<bool> = props.iter().map(|&p| c.is_public_method(p)).collect();
            let reduces = c.some_property_reduces_to_never(i);
            let cache = c
                .ty(i)
                .as_union_or_intersection_type()
                .property_cache_without_function_property_augment;
            (
                c.ty(d).flags.intersects(TypeFlags::UNION),
                c.ty(i).flags.intersects(TypeFlags::INTERSECTION),
                constituents.len(),
                c.sym(props[0]).value_declaration.is_some()
                    && c.sym(props[0]).value_declaration == c.sym(props[1]).value_declaration,
                public_methods,
                reduces,
                ["kind", "m"].map(|name| c.symbols.get(cache, name).is_some()),
            )
        });
        assert_eq!(
            got,
            (true, true, 3, true, vec![true, false], true, [true, false])
        );
    }

    /// `get_reduced_apparent_type` returns early for object types (plain,
    /// mapped, homomorphic mapped over an array) with the type that Go's
    /// `getReducedType(getApparentType(getReducedType(t)))` gives
    /// (checker.go:22251). A union property takes the value type of the
    /// index signature of each member that lacks the property and has one
    /// for the name's literal type (`a${string}` holds `"a"`, `x${string}`
    /// does not), with the literal type made once (Go
    /// `getApplicableIndexInfoForName`, checker.go:19352, per member).
    #[test]
    fn reduced_apparent_types_and_index_members_of_union_properties() {
        const SOURCE: &str = r#"
interface A { a: string }
interface B { [k: string]: number }
interface C { [k: string]: boolean }
interface D { [k: `x${string}`]: null }
interface E { [k: `a${string}`]: bigint }
type U = A | B | C | D | E;
type M = Record<"a" | "b", 1>;
type H<T> = { [K in keyof T]: T[K] };
type HA = H<string[]>;
type I = A & M;
"#;
        let got = with_alias_types(SOURCE, |c, types| {
            let same: Vec<bool> = types
                .iter()
                .map(|&t| {
                    let fast = c.get_reduced_apparent_type(t);
                    let reduced = c.get_reduced_type(t);
                    let apparent = c.get_apparent_type(reduced);
                    fast == c.get_reduced_type(apparent)
                })
                .collect();
            let u = types[0];
            let a = c.get_union_or_intersection_property(u, "a", false);
            let partial = c.sym(a).check_flags & CheckFlags::PARTIAL;
            let a_type = c.get_type_of_symbol(a);
            (same, partial, c.type_to_string(a_type))
        });
        assert_eq!(
            got,
            (
                vec![true; got.0.len()],
                CheckFlags::READ_PARTIAL | CheckFlags::WRITE_PARTIAL,
                "string | number | bigint | boolean".to_string(),
            )
        );
        assert!(got.0.len() >= 4);
    }

    /// `DeclarationSet::add` gives the list of Go `core.AppendIfUnique`
    /// (a scan per add), before and after the table takes over, for nodes
    /// of several files.
    #[test]
    fn declaration_set_keeps_the_append_if_unique_list() {
        let mut seed = 0x2545_f491_4f6c_dd1d_u64;
        let mut cases: Vec<Vec<Node>> = Vec::new();
        for (adds, distinct) in [
            (10, 4),
            (16, 16),
            (40, 16),
            (17, 17),
            (700, 300),
            (2000, 2000),
        ] {
            let nodes = (0..adds)
                .map(|_| {
                    seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                    let k = (seed >> 33) % distinct;
                    Node(((k % 3 + 1) << 32) | (k * 7 + 1))
                })
                .collect();
            cases.push(nodes);
        }
        // Every node distinct: the table is filled to its bound.
        cases.push((1..=64).map(|k| Node((1 << 32) | k)).collect());
        for nodes in cases {
            let mut want: Vec<Node> = Vec::new();
            for &node in &nodes {
                if !want.contains(&node) {
                    want.push(node);
                }
            }
            let mut set = DeclarationSet::default();
            let mut got: SmallVec<[Node; 4]> = SmallVec::new();
            for &node in &nodes {
                set.add(&mut got, node, nodes.len());
            }
            assert_eq!(got.as_slice(), want.as_slice());
        }
    }

    /// `DeclarationSet::add` gives the `core.AppendIfUnique` list when the
    /// list grows past `len_hint`: the table doubles before it is more than
    /// half full (a full table would loop forever). `Node::NIL`, which marks
    /// a free slot, takes the scan and is listed once.
    #[test]
    fn declaration_set_grows_past_its_hint() {
        let node = |k: u64| Node(((k % 3 + 1) << 32) | k);
        let mut nodes: Vec<Node> = (1..=300).map(node).collect();
        nodes.extend((1..=300).step_by(7).map(node));
        nodes.insert(40, Node::NIL);
        nodes.push(Node::NIL);
        let mut want: Vec<Node> = Vec::new();
        for &node in &nodes {
            if !want.contains(&node) {
                want.push(node);
            }
        }
        for len_hint in [0, 16, 40] {
            let mut set = DeclarationSet::default();
            let mut got: SmallVec<[Node; 4]> = SmallVec::new();
            for &node in &nodes {
                set.add(&mut got, node, len_hint);
                let used = set.slots.iter().filter(|slot| slot.is_some()).count();
                assert!(
                    used * 2 <= set.slots.len(),
                    "len_hint {len_hint}: {used} used"
                );
            }
            assert_eq!(got.as_slice(), want.as_slice(), "len_hint {len_hint}");
        }
    }

    /// `Node::NIL` added while the list is scanned, before the table takes
    /// over (R183 reviewer item 7): the fill puts each other listed node in
    /// the table, NIL stays listed once, and the later adds of NIL and of
    /// listed nodes (the first 20 again) push nothing. NIL as the add that fills the table
    /// (the 16th node) too. Go `core.AppendIfUnique` (core.go:380) lists a
    /// nil element like any other.
    #[test]
    fn declaration_set_lists_nil_added_before_the_table() {
        let node = |k: u64| Node(((k % 3 + 1) << 32) | k);
        for nil_at in [0, 3, DeclarationSet::SCAN - 1] {
            let mut nodes: Vec<Node> = (1..=100).map(node).collect();
            nodes.insert(nil_at, Node::NIL);
            nodes.extend([Node::NIL, node(50), Node::NIL, node(100)]);
            nodes.extend((1..=20).map(node));
            let mut want: Vec<Node> = Vec::new();
            for &node in &nodes {
                if !want.contains(&node) {
                    want.push(node);
                }
            }
            for len_hint in [0, 16, 200] {
                let mut set = DeclarationSet::default();
                let mut got: SmallVec<[Node; 4]> = SmallVec::new();
                for (i, &node) in nodes.iter().enumerate() {
                    set.add(&mut got, node, len_hint);
                    if i == DeclarationSet::SCAN {
                        assert!(!set.slots.is_empty(), "nil at {nil_at}: no table");
                    }
                }
                assert_eq!(
                    got.as_slice(),
                    want.as_slice(),
                    "nil at {nil_at}, len_hint {len_hint}"
                );
                let listed = got.iter().filter(|node| node.is_some()).count();
                let used = set.slots.iter().filter(|slot| slot.is_some()).count();
                assert_eq!(used, listed, "nil at {nil_at}, len_hint {len_hint}");
            }
        }
    }
}
