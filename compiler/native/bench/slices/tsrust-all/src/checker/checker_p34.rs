use crate::prelude::*;




#[cfg(test)]
mod tests {
    use super::*;

    /// Writes `aliases` as `type T<i> = <alias>;` lines of `a.ts` in a new
    /// project (`type T<i><alias>;` for an alias that starts with `<`, so it
    /// can have type parameters), with `strictNullChecks` set to `strict`,
    /// and runs `f` on the checker of `a.ts` with the type of each alias, in
    /// order.
    fn with_alias_types<R: Send + 'static>(
        label: &str,
        aliases: &[&str],
        strict: bool,
        f: impl FnOnce(&mut Checker, &[TypeId]) -> R + Send + 'static,
    ) -> R {
        let dir =
            std::env::temp_dir().join(format!("ts_goport_{label}_{strict}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source: String = aliases
            .iter()
            .enumerate()
            .map(|(i, alias)| {
                if alias.starts_with('<') {
                    format!("type T{i}{alias};\n")
                } else {
                    format!("type T{i} = {alias};\n")
                }
            })
            .collect();
        std::fs::write(dir.join("a.ts"), source).unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            format!(
                r#"{{ "compilerOptions": {{ "strictNullChecks": {strict}, "target": "es2020", "types": [] }}, "files": ["a.ts"] }}"#
            ),
        )
        .unwrap();
        let config = dir.join("tsconfig.json");
        let program = crate::program::try_load_version(&config.to_string_lossy(), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {}: {e}", config.display()));
        let _ = std::fs::remove_dir_all(&dir);
        let scope = crate::core::enter_program(Some(program));
        let file = program
            .source_files()
            .find(|file| file.info.file_name.ends_with("/a.ts"))
            .expect("a.ts is not in the program")
            .root;
        let result = crate::program::with_type_checker_for_file(file, move |checker| {
            let types: Vec<TypeId> = file
                .statements()
                .iter()
                .map(|alias| checker.get_type_from_type_node(alias.type_()))
                .collect();
            f(checker, &types)
        });
        drop(scope);
        crate::program::release_program(program);
        result
    }

    /// Go `getStringLiteralType` (checker.go:25770) keys its map on the
    /// value. The port keys `string_literal_types` on the value's FxHash and
    /// compares each type's value in the bucket (`checker_p28.rs`
    /// `get_string_literal_type_cow`, tmpl1). The test puts the type of "b"
    /// in the buckets of "a" and "c", as a hash collision would: "a" still
    /// gives its own type, and "c" makes a new type once.
    #[test]
    fn string_literal_types_compare_values_in_a_hash_bucket() {
        use std::hash::BuildHasher;
        let aliases = [r#""a""#, r#""b""#];
        let got = with_alias_types("strlit_bucket", &aliases, true, |checker, types| {
            let (a, b) = (types[0], types[1]);
            assert_eq!(checker.get_string_literal_type("a"), a);
            assert_eq!(checker.get_string_literal_type("b"), b);
            for value in ["a", "c"] {
                let hash = crate::rustc_hash::FxBuildHasher.hash_one(value);
                checker
                    .string_literal_types
                    .entry(hash)
                    .or_default()
                    .insert(0, b);
            }
            let c = checker.get_string_literal_type("c");
            (
                checker.get_string_literal_type("a") == a,
                c != b && c != a,
                checker.get_string_literal_value_ref(c) == "c",
                checker.get_string_literal_type("c") == c,
                checker.get_string_literal_type("b") == b,
            )
        });
        assert_eq!(got, (true, true, true, true, true));
    }

    /// Go `maybeTypeOfKind` recurses into a union or intersection
    /// constituent (checker.go:28071). unionfn1 tests the flags of other
    /// constituents in its loop.
    #[test]
    fn maybe_type_of_kind_looks_into_nested_constituents() {
        let aliases = [
            "(string & { a: 1 }) | { b: 1 }",
            "{ a: 1 } | { b: 1 }",
            "number | { b: 1 }",
        ];
        let got = with_alias_types("maybe_kind", &aliases, true, |checker, types| {
            types
                .iter()
                .map(|&t| {
                    (
                        checker.maybe_type_of_kind(t, TypeFlags::STRING),
                        checker.maybe_type_of_kind(t, TypeFlags::NUMBER),
                    )
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(got, [(true, false), (false, false), (false, true)]);
    }

    /// Go `getTypeFactsWorker` (checker.go:31466) and
    /// `getIntersectionTypeFacts` (:31602) for each arm, with and without
    /// `strictNullChecks`. unionfn1 split the switch in two and reads
    /// union and intersection constituents in its own loops.
    #[test]
    fn type_facts_match_go_arms() {
        type F = TypeFacts;
        let object = |strict: bool| {
            if strict {
                F::OBJECT_STRICT_FACTS
            } else {
                F::OBJECT_FACTS
            }
        };
        let function = |strict: bool| {
            if strict {
                F::FUNCTION_STRICT_FACTS
            } else {
                F::FUNCTION_FACTS
            }
        };
        // (alias, facts with strictNullChecks, facts without).
        let cases = [
            ("string", F::STRING_STRICT_FACTS, F::STRING_FACTS),
            (
                r#""" | "a""#,
                F::EMPTY_STRING_STRICT_FACTS | F::NON_EMPTY_STRING_STRICT_FACTS,
                F::EMPTY_STRING_FACTS | F::NON_EMPTY_STRING_FACTS,
            ),
            (
                "`a${string}`",
                F::NON_EMPTY_STRING_STRICT_FACTS,
                F::NON_EMPTY_STRING_FACTS,
            ),
            ("number", F::NUMBER_STRICT_FACTS, F::NUMBER_FACTS),
            (
                "0 | 1",
                F::ZERO_NUMBER_STRICT_FACTS | F::NON_ZERO_NUMBER_STRICT_FACTS,
                F::ZERO_NUMBER_FACTS | F::NON_ZERO_NUMBER_FACTS,
            ),
            ("bigint", F::BIG_INT_STRICT_FACTS, F::BIG_INT_FACTS),
            (
                "0n | 1n",
                F::ZERO_BIG_INT_STRICT_FACTS | F::NON_ZERO_BIG_INT_STRICT_FACTS,
                F::ZERO_BIG_INT_FACTS | F::NON_ZERO_BIG_INT_FACTS,
            ),
            ("boolean", F::BOOLEAN_STRICT_FACTS, F::BOOLEAN_FACTS),
            ("false", F::FALSE_STRICT_FACTS, F::FALSE_FACTS),
            ("true", F::TRUE_STRICT_FACTS, F::TRUE_FACTS),
            (
                r#""" | 0 | 0n | false | "a""#,
                F::EMPTY_STRING_STRICT_FACTS
                    | F::ZERO_NUMBER_STRICT_FACTS
                    | F::ZERO_BIG_INT_STRICT_FACTS
                    | F::FALSE_STRICT_FACTS
                    | F::NON_EMPTY_STRING_STRICT_FACTS,
                F::EMPTY_STRING_FACTS
                    | F::ZERO_NUMBER_FACTS
                    | F::ZERO_BIG_INT_FACTS
                    | F::FALSE_FACTS
                    | F::NON_EMPTY_STRING_FACTS,
            ),
            ("void", F::VOID_FACTS, F::VOID_FACTS),
            ("undefined", F::UNDEFINED_FACTS, F::UNDEFINED_FACTS),
            ("null", F::NULL_FACTS, F::NULL_FACTS),
            ("symbol", F::SYMBOL_STRICT_FACTS, F::SYMBOL_FACTS),
            ("object", object(true), object(false)),
            ("never", F::NONE, F::NONE),
            ("unknown", F::UNKNOWN_FACTS, F::UNKNOWN_FACTS),
            ("{}", F::EMPTY_OBJECT_STRICT_FACTS, F::EMPTY_OBJECT_FACTS),
            ("{ a: 1 }", object(true), object(false)),
            ("() => void", function(true), function(false)),
            // A primitive in an intersection: the object constituent is ignored.
            (
                r#"string & { __kind: "name" }"#,
                F::STRING_STRICT_FACTS,
                F::STRING_FACTS,
            ),
            // No primitive: facts are or-ed and and-ed by the two masks.
            (
                "{ a: 1 } & (() => void)",
                ((object(true) | function(true)) & F::OR_FACTS_MASK)
                    | (object(true) & function(true) & F::AND_FACTS_MASK),
                ((object(false) | function(false)) & F::OR_FACTS_MASK)
                    | (object(false) & function(false) & F::AND_FACTS_MASK),
            ),
            // The base constraint step: a constraint, and none (unknown).
            (
                r#"<X extends "" | 0> = X"#,
                F::EMPTY_STRING_STRICT_FACTS | F::ZERO_NUMBER_STRICT_FACTS,
                F::EMPTY_STRING_FACTS | F::ZERO_NUMBER_FACTS,
            ),
            ("<X> = X", F::UNKNOWN_FACTS, F::UNKNOWN_FACTS),
            // An intersection constituent of a union.
            (
                "(string & { a: 1 }) | number",
                F::STRING_STRICT_FACTS | F::NUMBER_STRICT_FACTS,
                F::STRING_FACTS | F::NUMBER_FACTS,
            ),
        ];
        let aliases: Vec<&str> = cases.iter().map(|case| case.0).collect();
        for strict in [true, false] {
            let got = with_alias_types("type_facts", &aliases, strict, |checker, types| {
                types
                    .iter()
                    .map(|&t| checker.get_type_facts(t, TypeFacts::ALL))
                    .collect::<Vec<_>>()
            });
            assert_eq!(got.len(), cases.len());
            for (case, got) in cases.iter().zip(got) {
                let want = if strict { case.1 } else { case.2 };
                assert_eq!(
                    got.0, want.0,
                    "facts of {} (strictNullChecks {strict})",
                    case.0
                );
            }
        }
    }

    /// Go `getIntersectionTypeFacts` (checker.go:31602) where chkfacts1's
    /// first walk takes some members and its second walk the others: a
    /// primitive after an object, a member that needs its constraint, and a
    /// caller that needs no object facts.
    #[test]
    fn intersection_type_facts_match_go() {
        type F = TypeFacts;
        let aliases = [
            r#"{ __kind: "name" } & string"#,
            "<X extends string> = { a: 1 } & X",
            "{ a: 1 } & { b: 1 }",
        ];
        for strict in [true, false] {
            let (object, string) = if strict {
                (F::OBJECT_STRICT_FACTS, F::STRING_STRICT_FACTS)
            } else {
                (F::OBJECT_FACTS, F::STRING_FACTS)
            };
            let both =
                ((object | string) & F::OR_FACTS_MASK) | (object & string & F::AND_FACTS_MASK);
            // Go's object arm gives none when the caller needs none of the
            // object facts.
            let possible = if strict {
                F::EMPTY_OBJECT_STRICT_FACTS | F::FUNCTION_STRICT_FACTS | F::OBJECT_STRICT_FACTS
            } else {
                F::EMPTY_OBJECT_FACTS | F::FUNCTION_FACTS | F::OBJECT_FACTS
            };
            let needs = F::EQ_UNDEFINED;
            let object_for_needs = if needs.intersects(possible) {
                object
            } else {
                F::NONE
            };
            let objects = ((object_for_needs & F::OR_FACTS_MASK)
                | (object_for_needs & F::AND_FACTS_MASK))
                & needs;
            let got = with_alias_types("isect_facts", &aliases, strict, move |checker, types| {
                vec![
                    checker.get_type_facts(types[0], F::ALL),
                    checker.get_intersection_type_facts(types[1], F::ALL),
                    // The base constraint of the intersection is
                    // `{ a: 1 } & string`.
                    checker.get_type_facts(types[1], F::ALL),
                    checker.get_type_facts(types[2], F::ALL),
                    checker.get_type_facts(types[2], needs),
                ]
            });
            assert_eq!(
                got,
                [string, both, string, object, objects],
                "strictNullChecks {strict}"
            );
        }
    }

    /// Go `getBaseConstraintOfType` (checker.go:27901) gives the same
    /// constraint before and after it is resolved (chkfacts1 reads a
    /// resolved one inline): a constraint, none, a circular one and an
    /// intersection (its own constraint).
    #[test]
    fn base_constraint_of_type_when_resolved() {
        let aliases = [
            "<X extends string> = X",
            "<X> = X",
            "<X extends Y, Y extends X> = X",
            "{ a: 1 } & { b: 2 }",
        ];
        let got = with_alias_types("base_constraint", &aliases, true, |checker, types| {
            let string_type = checker.string_type;
            types
                .iter()
                .map(|&t| {
                    let first = checker.get_base_constraint_of_type(t);
                    let second = checker.get_base_constraint_of_type(t);
                    assert_eq!(first, second);
                    if first.is_nil() {
                        "none"
                    } else if first == string_type {
                        "string"
                    } else if first == t {
                        "itself"
                    } else {
                        "other"
                    }
                })
                .collect::<Vec<_>>()
        });
        assert_eq!(got, ["string", "none", "none", "itself"]);
    }
}
