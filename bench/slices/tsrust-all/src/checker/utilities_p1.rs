//! Port of typescript-go `internal/checker/utilities.go` lines 1-903.
//!
//! PORT: Several Go `checker` package functions in this range have the same
//! snake name as an exported `ast` function that is already ported
//! (`ast.IsBinaryOperator`, `ast.IsEmptyObjectLiteral`,
//! `ast.EntityNameToString`, ...). Both modules are glob-exported through the
//! prelude, so a second `pub fn` with the same name would make every use
//! ambiguous. Those checker functions get a `checker_` prefix here (the same
//! choice `checker_p18.rs` made for `entityNameToString`).

use crate::prelude::*;
use std::cmp::Ordering;

// Go: checker/utilities.go:21 NewDiagnosticForNode

// Go: checker/utilities.go:31 NewDiagnosticChainForNode

// Go: checker/utilities.go:38 findInMap
// PORT: Go returns the zero value of V when nothing matches; here `V::default()`.
pub fn find_in_map<K, V: Default + Clone>(
    m: &FxHashMap<K, V>,
    mut predicate: impl FnMut(&V) -> bool,
) -> V {
    for value in m.values() {
        if predicate(value) {
            return value.clone();
        }
    }
    V::default()
}

// Go: checker/utilities.go:47 tokenIsIdentifierOrKeyword
pub fn token_is_identifier_or_keyword(token: SyntaxKind) -> bool {
    (token as u16) >= (SyntaxKind::Identifier as u16)
}

// Go: checker/utilities.go:51 tokenIsIdentifierOrKeywordOrGreaterThan
pub fn token_is_identifier_or_keyword_or_greater_than(token: SyntaxKind) -> bool {
    token == SyntaxKind::GreaterThanToken || token_is_identifier_or_keyword(token)
}

// Go: checker/utilities.go:55 hasOverrideModifier

// Go: checker/utilities.go:59 hasAsyncModifier

// Go: checker/utilities.go:63 getSelectedModifierFlags

// Go: checker/utilities.go:67 hasReadonlyModifier


// Go: checker/utilities.go:75 isEmptyObjectLiteral
// PORT: renamed; `is_empty_object_literal` is `ast.IsEmptyObjectLiteral` (same body).

// PORT: Go `type AssignmentKind int32` with its consts is `flags::AssignmentKind`
// (`AssignmentKind::NONE`, `DEFINITE`, `COMPOUND`). Go `AssignmentTarget` is a
// `*ast.Node` alias, so it is `Node`.

// Go: checker/utilities.go:89 getAssignmentTargetKind

// Go: checker/utilities.go:109 isDeleteTarget

// Go: checker/utilities.go:117 isInCompoundLikeAssignment

// Go: checker/utilities.go:122 isCompoundLikeAssignment

// Go: checker/utilities.go:127 isConstTypeReference
// PORT: renamed; `is_const_type_reference` is `ast.IsConstTypeReference` (same body).

// Go: checker/utilities.go:133 isConstTypeReferenceName
// isConstTypeReferenceName reports whether node is the `const` type name of a `const`
// assertion (`x as const` / `<const>x`), which must not be resolved as a real name.

// Go: checker/utilities.go:143 isExportAssignmentExpressionName
// isExportAssignmentExpressionName reports whether node is (the root entity name of) the
// expression of an `export =` / `export default` assignment. Referencing a namespace or
// type-only name there is legal, and checkExportAssignment decides whether it is an error,
// so checkIdentifier must not report a value-usage error for it.

// Go: checker/utilities.go:154 GetSingleVariableOfVariableStatement

// Go: checker/utilities.go:161 isTypeReferenceIdentifier

// Go: checker/utilities.go:168 IsInTypeQuery

// Go: checker/utilities.go:183 canHaveLocals


// Go: checker/utilities.go:201 isShorthandAmbientModule

// Go: checker/utilities.go:206 getAliasDeclarationFromName

// Go: checker/utilities.go:217 entityNameToString
// PORT: renamed; `entity_name_to_string` is `ast.EntityNameToString(name, getTextOfNode)`.

// Go: checker/utilities.go:221 getContainingQualifiedNameNode

// Go: checker/utilities.go:228 isSideEffectImport

// Go: checker/utilities.go:233 getExternalModuleRequireArgument

// Go: checker/utilities.go:240 isRightSideOfAccessExpression

// Go: checker/utilities.go:245 isTopLevelInExternalModuleAugmentation

// Go: checker/utilities.go:249 isSyntacticDefault


// Go: checker/utilities.go:260 isTypeAlias

// Go: checker/utilities.go:264 hasOnlyExpressionInitializer

// Go: checker/utilities.go:272 hasDotDotDotToken


// Go: checker/utilities.go:290 isJSDocOptionalParameter
pub fn is_js_doc_optional_parameter(node: Node) -> bool {
    false // !!!
}

// Go: checker/utilities.go:294 isExclamationToken

// Go: checker/utilities.go:298 isOptionalDeclaration


// PORT: Go `core.FindIndex(node.Parent.Parameters(), func(p) bool { return p == node })`
// used twice in isOptionalParameter. Returns -1 when not found, like Go.

// Go: checker/utilities.go:329 isEmptyArrayLiteral
// PORT: renamed; `is_empty_array_literal` is `ast.IsEmptyArrayLiteral` (same body).

// Go: checker/utilities.go:333 declarationBelongsToPrivateAmbientMember
// PORT: Go also has a `Checker` method with this name (checker.go:17940,
// ported in checker_p20.rs); methods and free functions do not clash in Rust.

// Go: checker/utilities.go:342 isPrivateWithinAmbient

// Go: checker/utilities.go:346 isTypeAssertion
// PORT: renamed; `is_type_assertion` is `ast.IsTypeAssertion`, which only
// checks for `KindTypeAssertionExpression`. This checker function differs:
// it skips parentheses and accepts any assertion expression.


/// Cached `compareSymbolsWorker` inputs for `sort_symbols`.
pub(crate) struct SymbolSortKey {
    pub(crate) symbol: SymbolId,
    has_declaration: bool,
    /// First declaration, or nil.
    declaration: Node,
    /// Source file of `declaration`.
    file: Node,
    /// `file_index_map[file]`, zero when absent (Go map miss).
    file_index: i32,
    pos: i32,
    /// A `u64` whose order is the order of the full comparator where two
    /// orders differ:
    /// - a declaration in a file of `file_index_map`: `(file_index, pos)`
    ///   packed so that `u64` order is `(file_index, pos)` order (below
    ///   2^63). Map indexes are unique and not negative, so two different
    ///   orders mean `compare_nodes` returns nonzero with the same sign:
    ///   `file_index` difference for different files, `pos` difference in
    ///   one file;
    /// - no declaration: `NO_DECLARATION_ORDER` and the first 63 bits of
    ///   the name's Go bytes (`go_bytes_prefix`). The full comparator puts a
    ///   symbol with a declaration first, which these orders, at 2^63 and
    ///   above, do too. Two symbols without a declaration compare by name
    ///   (Go `strings.Compare`), and different prefixes have its sign;
    /// - `NO_SORT_ORDER` for a nil symbol, a nil first declaration or a
    ///   file that is not in `file_index_map`.
    ///
    /// Equal orders (same declaration, same position, or names with the
    /// same prefix) fall through to the full comparator.
    // PERF (sortmisc1): on elysia-eden 86% of the `get_named_members`
    // comparisons (1.59 M of 1.85 M; Go makes 1.59 M of 1.99 M) are of two
    // symbols without a declaration (mapped type members), which the full
    // comparator orders by name text. The prefix decides all of them there.
    pub(crate) order: u64,
    /// Its text is read only when the orders tie.
    name: Name,
}

/// `SymbolSortKey::order` when the packed order does not apply. A
/// declaration order is below 2^63, and a name order is below this
/// (`go_bytes_prefix` never gives all one bits).
pub(crate) const NO_SORT_ORDER: u64 = u64::MAX;

/// The bit of `SymbolSortKey::order` that marks a symbol without a
/// declaration (its other bits are the name prefix).
const NO_DECLARATION_ORDER: u64 = 1 << 63;

// PORT: Go `strings.Compare` (byte order, returns -1/0/1). It compares the
// Go bytes, so lone surrogates and the internal symbol name prefix sort as in
// Go (see `compare_go_strings`).

// PORT: Go computes these differences as 64-bit `int`. Callers only use the
// sign, so a difference that does not fit in `i32` is clamped (sign kept).
fn clamp_compare(v: i64) -> i32 {
    v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

// PORT: Go `cmp.Compare` on `jsnum.Number` (a float64): NaN is less than any
// other value and equal to NaN; -0 equals +0.
fn compare_numbers(a: crate::jsnum::Number, b: crate::jsnum::Number) -> i32 {
    let (x, y) = (a.0, b.0);
    let x_nan = x.is_nan();
    let y_nan = y.is_nan();
    if x_nan {
        if y_nan {
            return 0;
        }
        return -1;
    }
    if y_nan {
        return 1;
    }
    if x < y {
        return -1;
    }
    if x > y {
        return 1;
    }
    0
}


// PERF: unionsort1. The union sort and the union searches decide most
// comparisons at the first steps of `compare_types`: the sort-order flags,
// the name, or the first value that the type's arm compares.
// `union_sort_key` packs those steps into one `u128` per type, read once per
// sort or per searched list, so most comparisons are one integer compare.
//
// The rule: when `union_sort_key(a) < union_sort_key(b)`, Go's
// `CompareTypes(a, b)` is negative, and Go returns at one of those first
// steps. Those steps have no effect: they come before every lazy symbol id
// (`compareSymbolsWorker` returns at `compareNodes` there) and before every
// case in which Go's order is not transitive (two files with the same index
// in `compareNodes`, an instantiation expression type against another
// object type). Equal keys say nothing, and `compare_types` decides. So a
// keyed comparison has the sign and the effects of Go's comparison, and a
// sort or search that uses it makes Go's comparisons in Go's order, with
// Go's result. The fields that the key reads are set when the type is made
// (Go reads the same fields when it compares).
//
// The key: `sort flags << 96 | unnamed << 95 | value` (value < 2^95).
// - sort flags: `getSortOrderFlags`, a `u32`.
// - unnamed: 1 when `getTypeNameSymbol` is nil (Go puts nil names last).
// - value, for a named type: the first 11 Go bytes of the name
//   (`go_bytes_prefix`). For an unnamed type, by the arm that it takes in
//   `compare_types`:
//   - any, unknown, string and the other types that only the id tells
//     apart: the type id (Go's last step);
//   - object, enum-like and unique symbol types: `symbol_sort_rank` of the
//     type's symbol (`compareSymbolsWorker` and the instantiation expression
//     branch both start with the first declarations);
//   - intersection: the member count (Go `compareTypeLists` compares the
//     lengths first), then the first 87 bits of the first member's key (the
//     first members are compared next; truncation keeps the order);
//   - string literal: the first 11 Go bytes of the value;
//   - number literal: `number_sort_value`;
//   - boolean literal: 0 for false, 1 for true;
//   - template literal: the first 11 Go bytes of the first text (Go
//     compares the text lists first, element by element);
//   - the other arms (union, bigint literal, index and so on): 0, so the
//     keys tie and `compare_types` decides.
// Equal sort flags give the same arm: types with equal flags take the same
// branch, and every enum-like unit type takes the enum branch.


/// Go `cmp.Compare` order of a number literal value: NaN first, -0 equal
/// to +0.
#[inline]
fn number_sort_value(n: crate::jsnum::Number) -> u64 {
    let x = n.0;
    if x.is_nan() {
        return 0;
    }
    // -0 + 0 is +0.
    let bits = (x + 0.0).to_bits();
    if bits >> 63 == 1 {
        !bits
    } else {
        bits | 1 << 63
    }
}

/// Lists shorter than this are sorted without keys: a key costs about as
/// much as a comparison, and a short list makes few comparisons.
const KEYED_SORT_MIN: usize = 8;

/// Longer lists are sorted by `sort_large_union_types`. A key pair is 8
/// times the size of a `TypeId`, and the sort moves its elements: on lists
/// of 10^4 to 10^6 types (eslint-plugin-svelte) a sort of the pairs ran
/// more instructions than the plain sort.
const KEYED_SORT_MAX: usize = 4096;

/// One distinct type of a large union sort (`sort_large_union_types`).
/// `first` and `second` are set for an unnamed intersection of 2 to 254
/// members: its first member, and the key of its second member.
#[derive(Clone, Copy)]
struct LargeSortEntry {
    key: u128,
    second: u128,
    first: TypeId,
    t: TypeId,
}

impl LargeSortEntry {
    /// The order of Go `CompareTypes(self.t, other.t)` when the entries
    /// tell it, else `Equal`. Different keys tell it (the PERF note above).
    /// Equal keys mean the same flags, both unnamed or both named, and for
    /// unnamed intersections the same member count. Then two intersections
    /// with the same first member get 0 for it in Go `compareTypeLists`,
    /// which goes on to the second members: different keys tell their
    /// order.
    #[inline]
    fn keyed_order(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key).then_with(|| {
            if self.first.is_some() && self.first == other.first {
                self.second.cmp(&other.second)
            } else {
                Ordering::Equal
            }
        })
    }
}


// PORT: Go `t.AsLiteralType().value.(string)`; panics like the Go type assertion.

// PORT: Go `t.AsLiteralType().value.(jsnum.Number)`.

// PORT: Go `t.AsLiteralType().value.(bool)`.

// PORT: Go `slices.Compare` on `[]string`: element-wise `strings.Compare`,
// then the shorter slice is less.

// Go: checker/utilities.go:624 getSortOrderFlags
// PORT: takes `&Type` (pure read of one type's flags); returns Go `int` as `i64`.



// Go: checker/utilities.go:668 compareTupleTypes
// PORT: Go takes `*TupleType`; here `&TupleType` borrowed from the type arena.
// Go pointer equality is `std::ptr::eq`.

// Go: checker/utilities.go:691 compareElementLabels


// PORT: the operator predicates below are renamed with a `checker_` prefix.
// `ast/fields.rs` already exports `is_exponentiation_operator`, ...,
// `is_binary_operator` (Go `ast.IsXxxOperator`, generated from the same kind
// sets), so the unprefixed names would be ambiguous through the prelude.

// Go: checker/utilities.go:1923 quotedAndCommaSeparated

// Go: checker/utilities.go:800 isExponentiationOperator
pub fn checker_is_exponentiation_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::AsteriskAsteriskToken
}

// Go: checker/utilities.go:804 isMultiplicativeOperator
pub fn checker_is_multiplicative_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::AsteriskToken
        || kind == SyntaxKind::SlashToken
        || kind == SyntaxKind::PercentToken
}

// Go: checker/utilities.go:808 isMultiplicativeOperatorOrHigher
pub fn checker_is_multiplicative_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_exponentiation_operator(kind) || checker_is_multiplicative_operator(kind)
}

// Go: checker/utilities.go:812 isAdditiveOperator
pub fn checker_is_additive_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::PlusToken || kind == SyntaxKind::MinusToken
}

// Go: checker/utilities.go:816 isAdditiveOperatorOrHigher
pub fn checker_is_additive_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_additive_operator(kind) || checker_is_multiplicative_operator_or_higher(kind)
}

// Go: checker/utilities.go:820 isShiftOperator
pub fn checker_is_shift_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::LessThanLessThanToken
        || kind == SyntaxKind::GreaterThanGreaterThanToken
        || kind == SyntaxKind::GreaterThanGreaterThanGreaterThanToken
}

// Go: checker/utilities.go:825 isShiftOperatorOrHigher
pub fn checker_is_shift_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_shift_operator(kind) || checker_is_additive_operator_or_higher(kind)
}

// Go: checker/utilities.go:829 isRelationalOperator
pub fn checker_is_relational_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::LessThanToken
        || kind == SyntaxKind::LessThanEqualsToken
        || kind == SyntaxKind::GreaterThanToken
        || kind == SyntaxKind::GreaterThanEqualsToken
        || kind == SyntaxKind::InstanceOfKeyword
        || kind == SyntaxKind::InKeyword
}

// Go: checker/utilities.go:834 isRelationalOperatorOrHigher
pub fn checker_is_relational_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_relational_operator(kind) || checker_is_shift_operator_or_higher(kind)
}

// Go: checker/utilities.go:838 isEqualityOperator
pub fn checker_is_equality_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::EqualsEqualsToken
        || kind == SyntaxKind::EqualsEqualsEqualsToken
        || kind == SyntaxKind::ExclamationEqualsToken
        || kind == SyntaxKind::ExclamationEqualsEqualsToken
}

// Go: checker/utilities.go:843 isEqualityOperatorOrHigher
pub fn checker_is_equality_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_equality_operator(kind) || checker_is_relational_operator_or_higher(kind)
}

// Go: checker/utilities.go:847 isBitwiseOperator
pub fn checker_is_bitwise_operator(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::AmpersandToken
        || kind == SyntaxKind::BarToken
        || kind == SyntaxKind::CaretToken
}

// Go: checker/utilities.go:851 isBitwiseOperatorOrHigher
pub fn checker_is_bitwise_operator_or_higher(kind: SyntaxKind) -> bool {
    checker_is_bitwise_operator(kind) || checker_is_equality_operator_or_higher(kind)
}

// Go: checker/utilities.go:855 isLogicalOperatorOrHigher
pub fn checker_is_logical_operator_or_higher(kind: SyntaxKind) -> bool {
    is_logical_binary_operator(kind) || checker_is_bitwise_operator_or_higher(kind)
}

// Go: checker/utilities.go:859 isAssignmentOperatorOrHigher
pub fn checker_is_assignment_operator_or_higher(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::QuestionQuestionToken
        || checker_is_logical_operator_or_higher(kind)
        || is_assignment_operator(kind)
}

// Go: checker/utilities.go:863 isBinaryOperator
pub fn checker_is_binary_operator(kind: SyntaxKind) -> bool {
    checker_is_assignment_operator_or_higher(kind) || kind == SyntaxKind::CommaToken
}


// Go: checker/utilities.go:871 isDeclarationReadonly

// orderedSetMapThreshold is the size at which an orderedSet materializes its dedup map.
// Below this, contains() scans the values slice.
// Go: checker/utilities.go:877 orderedSetMapThreshold
pub const ORDERED_SET_MAP_THRESHOLD: usize = 16;

// Go: checker/utilities.go:879 orderedSet
// PORT: Go nil map is `None`.
#[derive(Clone, Debug)]
pub struct OrderedSet<T: Eq + std::hash::Hash + Clone> {
    pub values_by_key: Option<FxHashSet<T>>,
    pub values: Vec<T>,
}

impl<T: Eq + std::hash::Hash + Clone> Default for OrderedSet<T> {
    fn default() -> Self {
        Self {
            values_by_key: None,
            values: Vec::new(),
        }
    }
}

impl<T: Eq + std::hash::Hash + Clone> OrderedSet<T> {
    // Go: checker/utilities.go:884 orderedSet.contains
    pub fn contains(&self, value: &T) -> bool {
        match &self.values_by_key {
            None => self.values.contains(value),
            Some(values_by_key) => values_by_key.contains(value),
        }
    }

    // Go: checker/utilities.go:892 orderedSet.add
    pub fn add(&mut self, value: T) {
        self.values.push(value.clone());
        // Small sets are served by a linear scan over values; only materialize the map once the set
        // grows large enough for hashing to win.
        if self.values_by_key.is_none() {
            if self.values.len() <= ORDERED_SET_MAP_THRESHOLD {
                return;
            }
            let mut values_by_key = FxHashSet::default();
            values_by_key.reserve(self.values.len());
            for v in &self.values[..self.values.len() - 1] {
                values_by_key.insert(v.clone());
            }
            self.values_by_key = Some(values_by_key);
        }
        if let Some(values_by_key) = &mut self.values_by_key {
            values_by_key.insert(value);
        }
    }
}

// Go: checker/utilities.go:908 getContainingFunctionOrClassStaticBlock

// Go: checker/utilities.go:912 isNodeDescendantOf
// PORT: renamed; `is_node_descendant_of` is `ast.IsNodeDescendantOf` (same body).


// Go: checker/utilities.go:941 isNumericLiteralName

// Go: checker/utilities.go:966 isThisProperty

#[cfg(test)]
pub(crate) mod union_sort_tests {
    use super::*;
    use crate::gostd::slices::{binary_search_func, sort_stable_func};
    use crate::scanner_util::go_string_from_bytes;
    use std::cmp::Ordering;

    /// Strings in the port form of Go strings: ASCII, long shared prefixes,
    /// NUL, non-ASCII, lone surrogate and invalid byte units, the internal
    /// symbol name prefix.
    fn strings() -> Vec<String> {
        let mut v: Vec<String> = [
            "",
            "a",
            "ab",
            "abc",
            "abcdefghijk",
            "abcdefghijkl",
            "abcdefghijkm",
            "abcdefghij",
            "abcdefghijk\u{0}",
            "\u{0}",
            "Z",
            "zz",
            "é",
            "aé",
            "a\u{7f}",
            "abcdefghijé",
            "abcdefghijkéz",
            "\u{FDD0}",
            crate::ast::INTERNAL_SYMBOL_NAME_PREFIX,
            "日本",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        for bytes in [
            &[0x61, 0xfe][..],
            &[0x61, 0x80, 0x62],
            &[0xed, 0xa0, 0x80],
            &[0x61, 0xed, 0xbf, 0xbf],
            &[0xff],
        ] {
            v.push(go_string_from_bytes(bytes.to_vec()));
        }
        v
    }

    /// The prefix order never contradicts Go `strings.Compare`.
    #[test]
    fn go_bytes_prefix_keeps_go_string_order() {
        let v = strings();
        let mut decided = 0;
        for a in &v {
            for b in &v {
                if go_bytes_prefix(a) < go_bytes_prefix(b) {
                    decided += 1;
                    assert_eq!(compare_go_strings(a, b), Ordering::Less, "{a:?} {b:?}");
                }
            }
        }
        assert!(decided > 200, "{decided}");
    }

    /// `number_sort_value` order is Go `cmp.Compare` order.
    #[test]
    fn number_sort_value_is_cmp_compare_order() {
        let v = [
            f64::NAN,
            f64::NEG_INFINITY,
            -1e300,
            -1.5,
            -f64::MIN_POSITIVE,
            -0.0,
            0.0,
            5e-324,
            1.0,
            1.5,
            1e300,
            f64::INFINITY,
        ];
        for &a in &v {
            for &b in &v {
                let (a, b) = (crate::jsnum::Number(a), crate::jsnum::Number(b));
                let c = compare_numbers(a, b);
                assert_eq!(
                    number_sort_value(a).cmp(&number_sort_value(b)),
                    c.cmp(&0),
                    "{a:?} {b:?}"
                );
            }
        }
    }

    /// The types of the type aliases in `source` (one file `a.ts`), then
    /// the members of those that are unions or intersections, on the
    /// checker of that file. Each call writes its files in a temp dir of
    /// its own, so no call reads or removes the files of another.
    pub(crate) fn with_alias_types<R: Send + 'static>(
        source: &str,
        f: impl FnOnce(&mut Checker, &[TypeId]) -> R + Send + 'static,
    ) -> R {
        static CALLS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let call = CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ts_goport_unionsort_{}_{call}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.ts"), source).unwrap();
        std::fs::write(
            dir.join("tsconfig.json"),
            r#"{ "compilerOptions": { "strict": true, "target": "es2020", "types": [] }, "files": ["a.ts"] }"#,
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
            let mut types: Vec<TypeId> = file
                .statements()
                .iter()
                .filter(|s| s.kind() == SyntaxKind::TypeAliasDeclaration)
                .map(|alias| checker.get_type_from_type_node(alias.type_()))
                .collect();
            // Then the members of the unions and intersections, which have
            // no alias.
            for i in 0..types.len() {
                if checker
                    .ty(types[i])
                    .flags
                    .intersects(TypeFlags::UNION_OR_INTERSECTION)
                {
                    for &m in checker.ty(types[i]).types().to_vec().iter() {
                        if !types.contains(&m) {
                            types.push(m);
                        }
                    }
                }
            }
            f(checker, &types)
        });
        drop(scope);
        crate::program::release_program(program);
        result
    }

    /// Members with a declaration (interfaces, a class, an object literal
    /// type) and mapped type members without one, whose names share long
    /// prefixes or are not ASCII.
    const SYMBOL_SOURCE: &str = r#"
interface I { b: 1; a: 1; abcdefghij1: 1; abcdefghij2: 1 }
interface J { a: 2; z: 2 }
class C { m() {} n = 1 }
type O = { q: 1; p: 2 }
type K = "b" | "a" | "abcdefghij1" | "abcdefghij2" | "abcdefgh" | "abcdefghX" | "abcdefg" | "abcdefgX" | "é" | "aé" | "Z" | "";
type M = { [P in K]: P };
type M2 = { [P in K | "extra" | "q"]: 1 };
type H = { [P in keyof I]: 2 };
type U = I & M & O;
"#;

    /// The `SymbolSortKey` order never contradicts `compareSymbolsWorker`,
    /// and `sort_symbol_sort_keys` gives Go's sort (`gostd` with
    /// `compare_symbols_worker`) on many orders of the symbols.
    #[test]
    fn symbol_sort_order_agrees_with_compare_symbols() {
        with_alias_types(SYMBOL_SOURCE, |c, types| {
            let mut symbols: Vec<SymbolId> = Vec::new();
            for &t in types {
                for &s in c.get_properties_of_type(t).iter() {
                    if !symbols.contains(&s) {
                        symbols.push(s);
                    }
                }
            }
            let mut last_file = (Node::NIL, 0);
            let keys: Vec<SymbolSortKey> = symbols
                .iter()
                .map(|&s| c.symbol_sort_key(s, &mut last_file))
                .collect();
            let mut decided = 0;
            let mut by_name = 0;
            let mut ties = 0;
            for a in &keys {
                for b in &keys {
                    if a.symbol == b.symbol {
                        continue;
                    }
                    assert!(a.order != NO_SORT_ORDER, "{:?}", a.name);
                    let s = c.compare_symbols_worker(a.symbol, b.symbol).signum();
                    match a.order.cmp(&b.order) {
                        Ordering::Less => assert_eq!(s, -1, "{:?} {:?}", a.name, b.name),
                        Ordering::Greater => assert_eq!(s, 1, "{:?} {:?}", a.name, b.name),
                        Ordering::Equal => ties += 1,
                    }
                    decided += usize::from(a.order != b.order);
                    by_name +=
                        usize::from(a.order != b.order && !a.has_declaration && !b.has_declaration);
                }
            }
            // Names that share their first 8 bytes tie (the order keeps 63
            // bits of them).
            let key = |name: &str| {
                keys.iter()
                    .find(|k| !k.has_declaration && &*k.name == name)
                    .unwrap_or_else(|| panic!("no member {name:?} without a declaration"))
                    .order
            };
            assert_eq!(key("abcdefghij1"), key("abcdefghij2"));
            assert_eq!(key("abcdefgh"), key("abcdefghX"));
            assert_ne!(key("abcdefg"), key("abcdefgX"));
            assert!(
                decided > 400 && by_name > 100 && ties > 0,
                "{decided} {by_name} {ties}"
            );
            let mut state = 0x9e37_79b9_7f4a_7c15u64;
            for round in 0..64 {
                let mut list: Vec<SymbolId> = symbols.clone();
                for i in (1..list.len()).rev() {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    list.swap(i, (state % (i as u64 + 1)) as usize);
                }
                let n = 2 + round % (list.len() - 1);
                list.truncate(n);
                let mut keyed = list.clone();
                c.sort_symbols(&mut keyed);
                let mut go = list;
                crate::gostd::slices::sort_func(&mut go, |&a, &b| c.compare_symbols_worker(a, b));
                assert_eq!(keyed, go, "round {round}");
            }
        });
    }

    /// Each arm of `compare_types`, and the ties: two interfaces with one
    /// name, references to one generic type, intersections with the same
    /// first member, string literals with a long shared prefix, objects
    /// with no symbol (tuples), instantiations of one anonymous type.
    const SOURCE: &str = r#"
interface A { a: 1 }
interface B { b: 1 }
interface Abcdefghijklmn1 { c: 1 }
interface Abcdefghijklmn2 { d: 1 }
namespace N1 { export interface Same { x: 1 } }
namespace N2 { export interface Same { y: 1 } }
enum E { X, Y, Z = "z" }
declare const s1: unique symbol;
declare const s2: unique symbol;
declare function f<T>(x: T): { v: T };
type T0 = string;
type T1 = number;
type T2 = boolean;
type T3 = any;
type T4 = unknown;
type T5 = never;
type T6 = undefined;
type T7 = null;
type T8 = void;
type T9 = object;
type T10 = bigint;
type T11 = symbol;
type T12 = "abcdefghijklmnop1";
type T13 = "abcdefghijklmnop2";
type T14 = "abc";
type T15 = "";
type T16 = "é";
type T17 = "aé";
type T18 = 1;
type T19 = -1;
type T20 = 0;
type T21 = 1.5;
type T22 = 1e300;
type T23 = true;
type T24 = false;
type T25 = 10n;
type T26 = -5n;
type T27 = A;
type T28 = B;
type T29 = N1.Same;
type T30 = N2.Same;
type T31 = A[];
type T32 = B[];
type T33 = Array<string>;
type T34 = Promise<A>;
type T35 = Promise<B>;
type T36 = [string];
type T37 = [number, string];
type T38 = readonly [string];
type T39 = { x: 1 };
type T40 = { y: 2 };
type T41 = () => void;
type T42 = A & B;
type T43 = A & { z: 1 };
type T44 = A & B & { z: 1 };
type T45 = string & { __brand: 1 };
type T46 = number & { __brand: 1 };
type T47 = E;
type T48 = E.X;
type T49 = E.Y;
type T50 = E.Z;
type T51 = typeof s1;
type T52 = typeof s2;
type T53 = `a${string}`;
type T54 = `b${number}`;
type T55 = `abcdefghijklmn${string}x`;
type T56 = `abcdefghijklmn${number}y`;
type T57 = keyof A;
type T58 = A["a"];
type T59 = Uppercase<string>;
type T60 = A | B;
type T61 = "a" | "b";
type T62 = string | number;
type T63 = Abcdefghijklmn1;
type T64 = Abcdefghijklmn2;
type T65 = ReturnType<typeof f<string>>;
type T66 = ReturnType<typeof f<number>>;
type T67 = Abcdefghijklmn1 & B;
type T68 = Abcdefghijklmn2 & B;
type T69 = Promise<A> | Promise<B> | Array<A> | Array<B> | [A] | [B, A];
type T70 = (A & B) | (A & { z: 1 }) | (Abcdefghijklmn1 & B) | (B & Abcdefghijklmn2);
type T71 = "abcdefghijklmnop1" | "abcdefghijklmnop2" | `x${string}` | `x${number}`;
type T72 = { x: 1 } | { y: 2 } | ReturnType<typeof f<string>> | ReturnType<typeof f<number>>;
"#;

    /// Keys never contradict `compare_types`, keys decide many pairs, and
    /// the ties named in `SOURCE` tie.
    #[test]
    fn union_sort_key_agrees_with_compare_types() {
        with_alias_types(SOURCE, |c, types| {
            let mut decided = 0;
            let mut ties = 0;
            for &a in types {
                for &b in types {
                    if a == b {
                        continue;
                    }
                    let (ka, kb) = (c.union_sort_key(a), c.union_sort_key(b));
                    let s = c.compare_types(a, b).signum();
                    match ka.cmp(&kb) {
                        Ordering::Less => assert_eq!(s, -1, "{a:?} {b:?} {ka:#x} {kb:#x}"),
                        Ordering::Greater => assert_eq!(s, 1, "{a:?} {b:?} {ka:#x} {kb:#x}"),
                        Ordering::Equal => ties += 1,
                    }
                    decided += usize::from(ka != kb);
                }
            }
            // The same name in two namespaces: equal keys.
            let key = |i: usize| c.union_sort_key(types[i]);
            assert_eq!(key(29), key(30));
            // Names and values that share their first 11 bytes tie.
            assert_eq!(key(63), key(64));
            assert_eq!(key(12), key(13));
            assert_ne!(key(14), key(15));
            assert!(decided > 3000 && ties > 0, "{decided} {ties}");
        });
    }

    /// The keyed sort and the keyed search give Go's results (`gostd` sort
    /// and search with `compare_types`) on many orders of the types, with
    /// duplicates.
    #[test]
    fn keyed_sort_and_search_match_go() {
        with_alias_types(SOURCE, |c, types| {
            let mut state = 0x9e37_79b9_7f4a_7c15u64;
            for round in 0..64 {
                // A deterministic shuffle with duplicates, of 8 to 80 types.
                let n = 8 + round % 73;
                let list: Vec<TypeId> = (0..n)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        types[(state % types.len() as u64) as usize]
                    })
                    .collect();
                let mut keyed = list.clone();
                c.sort_union_types(&mut keyed);
                let mut go = list.clone();
                sort_stable_func(&mut go, |&a, &b| c.compare_types(a, b));
                assert_eq!(keyed, go, "round {round}");
                go.dedup();
                let pairs = c.union_sort_keys(&go);
                for &t in types {
                    let want = binary_search_func(&go[..], t, |&a, &b| c.compare_types(a, b));
                    let key = c.union_sort_key(t);
                    assert_eq!(c.search_keyed_types(&pairs, key, t), want, "round {round}");
                    assert_eq!(c.search_union_types(&go, t), want, "round {round}");
                }
            }
        });
    }

    /// `SOURCE` with intersections like eslint-plugin-svelte's: unions of
    /// interfaces crossed with unions of interfaces, so many intersections
    /// share their first member, and names that share 11 bytes tie.
    const LARGE_SOURCE: &str = r#"
interface SvelteShorthandAttribute { k: 1 }
interface SvelteShorthandDirective { k: 2 }
interface SvelteProgram { k: 3 }
interface SvelteScriptElement { k: 4 }
interface Box<T> { v: T }
type U0 = (A | B | SvelteProgram | SvelteScriptElement) & (SvelteShorthandAttribute | SvelteShorthandDirective | Box<A> | Box<B> | { w: 1 });
type U1 = (A | SvelteProgram) & (B | Box<string>) & ({ p: 1 } | SvelteShorthandAttribute);
"#;

    /// Lists over `KEYED_SORT_MAX` (`sort_large_union_types`): when
    /// `keyed_order` tells an order it is the order of `compare_types`, it
    /// tells many pairs through the second members, and the sort gives Go's
    /// list for shuffles with many repeated types and tied keys.
    #[test]
    fn large_union_sort_matches_go() {
        with_alias_types(&format!("{SOURCE}{LARGE_SOURCE}"), |c, aliases| {
            // The aliases and the members of the union aliases.
            let mut types = aliases.to_vec();
            for &t in aliases {
                if c.ty(t).flags.intersects(TypeFlags::UNION) {
                    types.extend_from_slice(c.ty(t).types());
                }
            }
            types.sort_unstable();
            types.dedup();
            let (mut second, mut ties) = (0, 0);
            for &a in &types {
                for &b in &types {
                    if a == b {
                        continue;
                    }
                    let (ea, eb) = (c.large_sort_entry(a), c.large_sort_entry(b));
                    let s = c.compare_types(a, b).signum();
                    match ea.keyed_order(&eb) {
                        Ordering::Less => assert_eq!(s, -1, "{a:?} {b:?}"),
                        Ordering::Greater => assert_eq!(s, 1, "{a:?} {b:?}"),
                        Ordering::Equal => ties += 1,
                    }
                    second += usize::from(ea.key == eb.key && ea.keyed_order(&eb).is_ne());
                }
            }
            assert!(second > 40 && ties > 0, "{second} {ties}");
            let mut state = 0x2545_f491_4f6c_dd1du64;
            for n in [KEYED_SORT_MAX + 1, 3 * KEYED_SORT_MAX + 7] {
                let list: Vec<TypeId> = (0..n)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        types[(state % types.len() as u64) as usize]
                    })
                    .collect();
                let mut large = list.clone();
                c.sort_union_types(&mut large);
                let mut go = list;
                sort_stable_func(&mut go, |&a, &b| c.compare_types(a, b));
                assert_eq!(large, go, "n {n}");
            }
        });
    }

    /// `contains_types_by_entries` (unionsub1) gives Go's answer: each
    /// source searched with `compare_types` in Go's order of the targets,
    /// up to the first miss. The types are those of
    /// `large_union_sort_matches_go`; the targets lack some of them.
    #[test]
    fn entry_searches_match_go() {
        with_alias_types(&format!("{SOURCE}{LARGE_SOURCE}"), |c, aliases| {
            let mut types = aliases.to_vec();
            for &t in aliases {
                if c.ty(t).flags.intersects(TypeFlags::UNION) {
                    types.extend_from_slice(c.ty(t).types());
                }
            }
            types.sort_unstable();
            types.dedup();
            sort_stable_func(&mut types, |&a, &b| c.compare_types(a, b));
            let go = |targets: &[TypeId], sources: &[TypeId]| {
                sources
                    .iter()
                    .all(|&t| binary_search_func(targets, t, |&a, &b| c.compare_types(a, b)).1)
            };
            let mut state = 0x9e37_79b9_7f4a_7c15u64;
            let (mut found, mut missed) = (0, 0);
            for round in 0..64 {
                let (mut targets, mut sources) = (Vec::new(), Vec::new());
                for &t in &types {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    if round % 2 == 0 || state % 16 != 0 {
                        targets.push(t);
                    }
                    if state % 3 == 0 {
                        sources.push(t);
                    }
                }
                let want = go(&targets, &sources);
                assert_eq!(
                    c.contains_types_by_entries(&targets, &sources),
                    want,
                    "round {round}"
                );
                if want {
                    found += 1
                } else {
                    missed += 1
                }
                for &t in &types {
                    let want = go(&targets, &[t]);
                    assert_eq!(c.contains_types_by_entries(&targets, &[t]), want, "{t:?}");
                }
            }
            assert!(found > 0 && missed > 0, "{found} {missed}");
        });
    }

    /// Lists over `KEYED_SORT_MAX`: the large sort makes the comparisons of
    /// Go's sort that have an effect, in Go's order. The one effect of
    /// `compare_types` is a lazy symbol id: `compare_symbols_worker` falls
    /// back to ids for two symbols with no declaration and one name (Go
    /// makes such symbols for import attributes, checker.go:5582). So the
    /// `LARGE_SOURCE` types get 24 anonymous types of such symbols, and the
    /// intersections of each with `A`, and the two sorts of one shuffle,
    /// each on its own checker, give the same list and the same ids to the
    /// same symbols, in the same order.
    #[test]
    fn large_union_sort_assigns_go_symbol_ids() {
        let run = |large: bool| {
            with_alias_types(&format!("{SOURCE}{LARGE_SOURCE}"), move |c, aliases| {
                let mut types = aliases.to_vec();
                for &t in aliases {
                    if c.ty(t).flags.intersects(TypeFlags::UNION) {
                        types.extend_from_slice(c.ty(t).types());
                    }
                }
                types.sort_unstable();
                types.dedup();
                let a = aliases[0];
                let bare: Vec<SymbolId> = (0..24)
                    .map(|_| {
                        c.new_symbol(
                            SymbolFlags::TYPE_LITERAL,
                            crate::ast::INTERNAL_SYMBOL_NAME_TYPE,
                        )
                    })
                    .collect();
                for &symbol in &bare {
                    let t = c.new_anonymous_type(symbol, SymbolTable::NIL, &[], &[], &[]);
                    let both = c.get_intersection_type(&[t, a]);
                    types.extend([t, both]);
                }
                let mut state = 0x6a09_e667_f3bc_c908u64;
                let mut list: Vec<TypeId> = (0..=KEYED_SORT_MAX)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        types[(state % types.len() as u64) as usize]
                    })
                    .collect();
                let first = crate::ast::next_ids().1;
                if large {
                    c.sort_union_types(&mut list);
                } else {
                    sort_stable_func(&mut list, |&x, &y| c.compare_types(x, y));
                }
                let last = crate::ast::next_ids().1;
                // The id that the sort gave each bare symbol, from 1 in the
                // order the sort gave them; 0 for none.
                let ids: Vec<u64> = bare
                    .iter()
                    .map(|&symbol| {
                        let id = crate::ast::get_symbol_id(&c.symbols, symbol);
                        if id <= last { id - first } else { 0 }
                    })
                    .collect();
                let order: Vec<usize> = list
                    .iter()
                    .map(|t| types.iter().position(|x| x == t).unwrap())
                    .collect();
                (order, last - first, ids)
            })
        };
        let (large, go) = (run(true), run(false));
        assert!(go.1 >= 12, "Go's sort gave {} symbol ids", go.1);
        assert_eq!(large.1, go.1, "symbol ids given");
        assert_eq!(large.2, go.2, "the ids of the bare symbols");
        assert_eq!(large.0, go.0, "the order");
    }
}
