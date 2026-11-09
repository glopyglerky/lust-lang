use crate::prelude::*;

// Port of typescript-go `internal/checker/relater.go` lines 2796-3902.
//
// PORT: Go `*Relater` methods hold `r.c *Checker`. A Rust `Relater` cannot
// hold `&mut Checker`, and Go passes `r.isRelatedToWorker` as a stored
// `TypeComparer` (for example to `newInferenceContext`) that is called again
// while this relater is in use. So every `(r *Relater)` method is ported as
// an `impl Checker` method that takes the relater handle
// `r: &Rc<RefCell<Relater>>` as its first parameter (same snake name; Go has
// no Checker method with any of these names). `r.c.X` becomes `self.X`.
// Relater fields are read and written through short `r.borrow()` /
// `r.borrow_mut()` calls that never live across another call, so re-entrant
// use of the same relater is safe.

/// `is_type_subset_of_union` keeps the answer for a union source when its
/// searches make about this many comparisons or more. Shorter calls cost
/// about as much as a lookup, and the table stays small.
const UNION_SUBSET_MEMO_MIN: usize = 64;





#[cfg(test)]
mod union_subset_tests {
    use super::*;
    use crate::checker::utilities_p1::union_sort_tests::with_alias_types;

    /// `is_type_subset_of_union` gives Go's answer (each source member
    /// searched in the target) on first and repeat calls, and keeps only
    /// the answers of long searches. The first calls from `Huge` or `Most`
    /// (300 and 260 types) to a target of 20 types or more search on
    /// entries (`contains_types_by_entries`).
    #[test]
    fn kept_subset_answers_are_go_answers() {
        let literals = |r: std::ops::Range<usize>| {
            r.map(|i| format!("\"a{i}\""))
                .collect::<Vec<_>>()
                .join(" | ")
        };
        let numbers =
            |r: std::ops::Range<usize>| r.map(|i| i.to_string()).collect::<Vec<_>>().join(" | ");
        let source = format!(
            "type Big = {};\ntype Half = {};\ntype Other = {} | \"zz\";\ntype Few = \"a1\" | \"a2\";\n\
             type Obj = {{ x: 1 }} | {{ y: 2 }} | {};\ntype Huge = {} | {};\ntype Most = {} | {};\n",
            literals(0..40),
            literals(0..20),
            literals(5..25),
            literals(0..12),
            literals(0..240),
            numbers(0..60),
            literals(0..200),
            numbers(0..60),
        );
        with_alias_types(&source, |c, types| {
            let unions: Vec<TypeId> = types
                .iter()
                .copied()
                .filter(|&t| c.ty(t).flags.intersects(TypeFlags::UNION))
                .collect();
            let [big, half, other, few, obj, huge, most] = unions[..] else {
                panic!("{} unions", unions.len());
            };
            for round in 0..2 {
                for &s in &unions {
                    for &t in &unions {
                        let targets = c.ty(t).types();
                        let go = c
                            .ty(s)
                            .types()
                            .iter()
                            .all(|&m| c.search_union_types(targets, m).1);
                        assert_eq!(c.is_type_subset_of_union(s, t), go, "round {round}");
                    }
                }
            }
            let kept = |s, t| c.union_subset_answers.get(&(s, t)).copied();
            assert_eq!(kept(half, big), Some(true));
            assert_eq!(kept(other, big), Some(false));
            assert_eq!(kept(big, obj), Some(false));
            assert_eq!(kept(obj, big), Some(false));
            assert_eq!(kept(most, huge), Some(true));
            assert_eq!(kept(huge, most), Some(false));
            assert_eq!(kept(most, half), Some(false));
            // 2 searches of 6 steps: not kept.
            assert_eq!(kept(few, big), None);
            assert!(c.is_type_subset_of_union(few, big));
        });
    }
}
