//! Port of typescript-go `checker/relater.go` lines 1827-2795.

use crate::prelude::*;
use crate::smallvec::SmallVec;
use std::borrow::Cow;


// Go: checker/relater.go:2559 visibilityToString
pub fn visibility_to_string(flags: ModifierFlags) -> String {
    if flags == ModifierFlags::PRIVATE {
        return "private".to_string();
    }
    if flags == ModifierFlags::PROTECTED {
        return "protected".to_string();
    }
    "public".to_string()
}

// Go: checker/relater.go:2569 errorState



// Go: checker/relater.go:2574 ErrorChain
// PORT: Go `*ErrorChain` nodes are immutable once built and compared by
// pointer, so they are `Rc<ErrorChain>` (nil is `None`; compare with
// `Rc::ptr_eq`). Go `args []any` are diagnostic args, so `Vec<String>`.

// Go: checker/relater.go:2580 Relater
// PORT: The Go `c *Checker` field is not stored. Every `(r *Relater)` method
// is an `impl Checker` method that takes the handle `r: &Rc<RefCell<Relater>>`
// right after `self` (same convention as relater_p4/p5). Borrows of `r` are
// kept short and never held across a checker call. Go `relation` is `kind`
// (see `RelationKind`); a pooled relater keeps the last kind in place of Go
// nil. `next` links the free list in `Checker::free_relater`.

// PORT: perf. Up to this many maybe keys, membership is a linear scan of
// `maybe_keys` and `maybe_keys_set` stays empty. The stack is almost always
// this small, and the scan is cheaper than the hash probe and the set writes.
const MAYBE_KEYS_SCAN_LIMIT: usize = 16;

// PORT: perf. A set that grew past this capacity is dropped when the stack
// becomes small again, because hashbrown never shrinks and `clear` costs time
// in its capacity.
const MAYBE_KEYS_SET_KEEP_CAPACITY: usize = 256;

// Empties the maybe keys set. Drops a large allocation instead of keeping it,
// so an empty set never holds more than the keep capacity.




/// One Go `addMatch(s, p)` call of `inferFromLiteralPartsToTemplateLiteral`:
/// the source text from part `seg` at byte `pos` to part `s` at byte `p`.
/// The byte offsets are in the Go bytes of the texts.
#[derive(Clone, Copy)]
struct LiteralPartMatch {
    seg: usize,
    pos: usize,
    s: usize,
    p: usize,
}

type LiteralPartMatches = SmallVec<[LiteralPartMatch; 4]>;

/// The placeholder types that `infer_types_from_template_literal_type` infers.
pub type TemplateLiteralInferences = SmallVec<[TypeId; 4]>;



/// The start and end texts (Go bytes) of the template literals in a list of
/// template and string mapping types, read once, so that each string
/// literal of a union is tested against them without reading each template
/// again (`template_candidates_of_string_literal`).
// PERF (sortmisc1): no Go type. Go `removeStringLiteralsMatchedByTemplateLiterals`
// tests every (string literal, template) pair, and almost every pair fails
// this test (immich: 22.3 M pairs, none passed).
pub(crate) struct TemplateLiteralEnds {
    /// The start text and then the end text of each template literal.
    bytes: Vec<u8>,
    /// For each type of the list: the offset of its start text in `bytes`
    /// and the lengths of its start and end texts, or `None` for a string
    /// mapping.
    spans: Vec<Option<(usize, usize, usize)>>,
}

impl TemplateLiteralEnds {
}


// PORT: perf. The text matching of Go `inferFromLiteralPartsToTemplateLiteral`
// without the type creation. It records each `addMatch` call in `matches`, so
// the caller can keep the source text borrowed from the checker while it
// matches and make the types after. It returns false where Go returns nil.
// The matches recorded before a failure stay in `matches`, because Go has
// already made their types at that point. Go matches the bytes of the
// strings, so the texts are the Go bytes of the port forms.

#[cfg(test)]
mod template_ends_tests {
    use super::*;
    use crate::checker::utilities_p1::union_sort_tests::with_alias_types;

    /// String literals and template literal types whose start and end texts
    /// share bytes, are empty, are longer than the string, or are not ASCII,
    /// and a string mapping type.
    const SOURCE: &str = r#"
type S1 = "a.b";
type S2 = "x";
type S3 = "abc";
type S4 = "";
type S5 = "é.z";
type S6 = "aXz";
type S7 = "123.b";
type S8 = "ab";
type T1 = `${number}.b`;
type T2 = `a.${string}`;
type T3 = `${string}z`;
type T4 = `a${string}b${string}c`;
type T5 = `${string}`;
type T6 = `é${string}`;
type T7 = `ab${string}ab`;
type T8 = `${string}.${string}`;
type T9 = Uppercase<string>;
"#;

    /// `template_candidates_of_string_literal` gives, in list order, every
    /// string mapping and every template literal that
    /// `string_literal_misses_template_literal_ends` does not reject, and so
    /// every template that matches the string.
    #[test]
    fn template_candidates_are_the_templates_that_pass_the_end_test() {
        with_alias_types(SOURCE, |c, types| {
            let strings: Vec<TypeId> = types
                .iter()
                .copied()
                .filter(|&t| c.ty(t).flags.intersects(TypeFlags::STRING_LITERAL))
                .collect();
            let templates: Vec<TypeId> = types
                .iter()
                .copied()
                .filter(|&t| {
                    c.is_pattern_literal_type(t)
                        || c.ty(t).flags.intersects(TypeFlags::STRING_MAPPING)
                })
                .collect();
            // `${string}` is `string`, not a template.
            assert_eq!((strings.len(), templates.len()), (8, 8));
            let ends = TemplateLiteralEnds::new(c, &templates);
            let (mut kept, mut matched) = (0, 0);
            for &s in &strings {
                let mut candidates = SmallVec::new();
                c.template_candidates_of_string_literal(s, &templates, &ends, &mut candidates);
                let want: Vec<TypeId> = templates
                    .iter()
                    .copied()
                    .filter(|&t| {
                        !c.ty(t).flags.intersects(TypeFlags::TEMPLATE_LITERAL)
                            || !c.string_literal_misses_template_literal_ends(s, t)
                    })
                    .collect();
                assert_eq!(
                    &candidates[..],
                    &want[..],
                    "{:?}",
                    c.get_string_literal_value_ref(s)
                );
                kept += candidates.len();
                for &t in &templates {
                    if c.is_type_matched_by_template_literal_or_string_mapping(s, t) {
                        matched += 1;
                        assert!(
                            candidates.contains(&t),
                            "{:?}",
                            c.get_string_literal_value_ref(s)
                        );
                    }
                }
            }
            assert!(
                kept > matched && matched > 5 && kept < strings.len() * templates.len(),
                "{kept} {matched}"
            );
        });
    }
}

#[cfg(test)]
mod related_info_tests {
    use super::*;

    /// A saved `RelatedInfo` (an error state) keeps the list it saw when the
    /// relater pushes after the save, and after a restore and a new push, as
    /// Go's `errorState` keeps its slice header (relater.go:3212, :3219).
    #[test]
    fn a_saved_state_keeps_its_related_info() {
        let message = diag::X_0_is_declared_here;
        let d = |name: &str| {
            new_diagnostic(
                Node::NIL,
                TextRange::new(0, 0),
                message,
                vec![name.to_string()],
            )
        };
        let names = |info: &RelatedInfo| -> Vec<String> {
            info.as_slice()
                .iter()
                .map(|d| d.message_args[0].clone())
                .collect()
        };
        let mut info = RelatedInfo::default();
        let empty = info.clone();
        info.push(d("a"));
        let saved = info.clone();
        info.push(d("b"));
        assert_eq!(
            (names(&empty), names(&saved), names(&info)),
            (vec![], vec!["a".to_string()], vec!["a".into(), "b".into()])
        );
        // Restore, then push again: the saved state and the dropped list do
        // not change.
        let dropped = info;
        info = saved.clone();
        info.push(d("c"));
        assert_eq!(
            (names(&saved), names(&dropped), names(&info)),
            (
                vec!["a".to_string()],
                vec!["a".into(), "b".into()],
                vec!["a".into(), "c".into()]
            )
        );
        info.clear();
        assert!(info.as_slice().is_empty());
    }
}
