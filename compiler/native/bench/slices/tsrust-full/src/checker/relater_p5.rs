use crate::prelude::*;


// PORT: Go methods on `*Relater` are `impl Checker` methods with the Go snake
// name that take the relater handle `r: &Rc<RefCell<Relater>>` right after
// `self`. The relater is shared (`Checker::free_relater` pools it) and Go
// passes `r.reportError` and `r.isRelatedTo` as func values into
// `compareSignaturesRelated` (a 'static `TypeComparer`), so each borrow of the
// relater is kept short and never held across a checker call. Go `r.c` is
// `self`. Go `r.relation == r.c.xxxRelation` compares the relater's
// `RelationKind`.
// PORT: Go `*ErrorChain` is `Option<Rc<ErrorChain>>` (the chain is shared by
// saved error states). `ErrorChain { next, message, args: Vec<String> }`.

// PORT: Go compares `*diagnostics.Message` pointers. `diag::X` are statics, so
// pointer equality is exact.


// Go: checker/relater.go:4912 addToDottedName


// PORT: Go takes `arg any` and asserts a string; diagnostic args are
// `String` in this port.
// Go: checker/relater.go:4959 getPropertyNameArg
pub fn get_property_name_arg(arg: &str) -> String {
    let s = arg.as_bytes();
    if !s.is_empty() && (s[0] == b'"' || s[0] == b'\'' || s[0] == b'`') {
        return "[".to_string() + arg + "]";
    }
    arg.to_string()
}

// Go: checker/relater.go:4967 isConversionOrInterfaceImplementationMessage

// Go: checker/relater.go:4976 chainDepth


#[cfg(test)]
mod distribution_dependent_tests {
    use super::*;
    use crate::checker::utilities_p1::union_sort_tests::with_alias_types;

    /// `is_distribution_dependent` gives the answer of Go's walk
    /// (relater.go:5026, checker.go:22823) on the first call and on a repeat,
    /// and the root keeps it after the first call. A root that is not
    /// distributive keeps nothing. The roots: `T` in the true type, `T` in
    /// the false type, `T` in neither, a `typeof` of a value outside the
    /// alias (resolved by the first walk), and `[T]`, not distributive.
    #[test]
    fn the_root_keeps_the_distribution_dependence() {
        const SOURCE: &str = r#"
declare const v: string;
type A<T> = T extends string ? T[] : never;
type B<T> = T extends string ? number : { x: T };
type C<T> = T extends string ? number : boolean;
type D<T> = T extends string ? typeof v : 0;
type E<T> = [T] extends [string] ? T : never;
"#;
        let got = with_alias_types(SOURCE, |c, types| {
            types
                .iter()
                .map(|&t| {
                    let root = c.ty(t).as_conditional_type().root.clone();
                    let before = root.borrow().distribution_dependent;
                    let first = c.is_distribution_dependent(&root);
                    let kept = root.borrow().distribution_dependent;
                    let (check_type, node) = {
                        let rb = root.borrow();
                        (rb.check_type, rb.node)
                    };
                    let walk = c
                        .is_type_parameter_possibly_referenced(check_type, node.true_type())
                        || c.is_type_parameter_possibly_referenced(check_type, node.false_type());
                    let repeat = c.is_distribution_dependent(&root);
                    (before, first, kept, walk, repeat)
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(
            got,
            vec![
                (None, true, Some(true), true, true),
                (None, true, Some(true), true, true),
                (None, false, Some(false), false, false),
                (None, false, Some(false), false, false),
                (None, false, None, true, false),
            ]
        );
    }
}
