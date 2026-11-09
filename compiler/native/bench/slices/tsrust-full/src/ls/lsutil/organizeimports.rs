use crate::ls::lsutil::prelude::*;

use crate::frontend::core_binarysearch::binary_search_unique_func;
use crate::frontend::stringutil_ls;
use crate::gostd::norm;
use crate::gostd::unicode;
use crate::gostd::unicode_tables::{self, RangeTable};

// Port of Go `ls/lsutil/organizeimports.go`.

// PORT: Go `func(a, b string) int` values. Go stores them in slices, maps and
// structs and compares them only with nil; a nil comparer is `None`. The
// unicode comparers are closures, so this is a shared `dyn Fn` (PORTING:
// stored funcs are `Rc<dyn Fn>`).
pub type StringComparer = Rc<dyn Fn(&str, &str) -> i32>;

// PORT: Go `func(s1, s2 *ast.Node) int` values returned to callers.
pub type NodeComparer = Rc<dyn Fn(Node, Node) -> i32>;

// Go: ls/lsutil/organizeimports.go:18 FilterImportDeclarations
// FilterImportDeclarations filters out non-import declarations from a list of statements.

// Go: ls/lsutil/organizeimports.go:25 GetDetectionLists
// GetDetectionLists returns the lists of comparers and type orders to test for organize imports detection.

// Go: ls/lsutil/organizeimports.go:50 ResolveOrganizeImportsSort

// Go: ls/lsutil/organizeimports.go:76 getOrganizeImportsOrdinalStringComparer

// Go: ls/lsutil/organizeimports.go:83 getOrganizeImportsNaturalStringComparer

// Go: ls/lsutil/organizeimports.go:89 getOrganizeImportsUnicodeStringComparer

// Go: ls/lsutil/organizeimports.go:99 compareOrganizeImportsNaturalStrings

// Go: ls/lsutil/organizeimports.go:113 compareOrganizeImportsUnicodeStrings

// Go: ls/lsutil/organizeimports.go:133 naturalCollationKey

// Go: ls/lsutil/organizeimports.go:137 removeDiacritics
// PORT: Go `strings.Map` over `norm.NFD.String(s)`. The NFD form of a valid
// UTF-8 string is valid UTF-8, so the lossy conversion never replaces.

// Go: ls/lsutil/organizeimports.go:146 compareOrganizeImportsUnicodeKeys

// Go: ls/lsutil/organizeimports.go:153 compareStringsNumeric
// PORT: Go decodes one rune with `utf8.DecodeRuneInString`; a `&str` is
// valid UTF-8, so its first `char` is that rune.

// Go: ls/lsutil/organizeimports.go:181 isASCIIDigit

// Go: ls/lsutil/organizeimports.go:185 asciiDigitRunEnd

// Go: ls/lsutil/organizeimports.go:193 compareNumericText

// Go: ls/lsutil/organizeimports.go:212 compareOrganizeImportsCaseUpperFirst

// Go: ls/lsutil/organizeimports.go:216 compareOrganizeImportsCase

// Go: unicode/letter.go:163 Is
/// Go `unicode.Is`: reports whether the rune is in the table of ranges.
/// PORT: the same scan as `gostd::collate`; Go's linear and binary searches
/// give the same answer.
fn unicode_is(range_tab: &RangeTable, r: u32) -> bool {
    let r16 = range_tab.r16;
    if let Some(&(_, hi, _)) = r16.last()
        && r <= u32::from(hi)
    {
        return r16.iter().any(|&(lo, hi, stride)| {
            let (lo, hi, stride) = (u32::from(lo), u32::from(hi), u32::from(stride));
            lo <= r && r <= hi && (stride == 1 || (r - lo) % stride == 0)
        });
    }
    let r32 = range_tab.r32;
    if let Some(&(lo, _, _)) = r32.first()
        && r >= lo
    {
        return r32.iter().any(|&(lo, hi, stride)| {
            lo <= r && r <= hi && (stride == 1 || (r - lo) % stride == 0)
        });
    }
    false
}

// PORT: Go `strings.ToLower` maps each rune with `unicode.ToLower`. Rust
// `str::to_lowercase` uses the full mapping of a newer Unicode and the
// final-sigma rule, which Go does not.

// Go: ls/lsutil/organizeimports.go:248 getOrganizeImportsPresetStringComparer

// Go: ls/lsutil/organizeimports.go:261 getOrganizeImportsStringComparer

// Go: ls/lsutil/organizeimports.go:271 getModuleSpecifierExpression

// Go: ls/lsutil/organizeimports.go:299 GetExternalModuleName
// GetExternalModuleName returns the module name from a module specifier expression.

// Go: ls/lsutil/organizeimports.go:307 CompareModuleSpecifiers
// CompareModuleSpecifiers compares two module specifiers using the given comparer.

// Go: ls/lsutil/organizeimports.go:319 compareImportKind

// getImportKindOrder returns the sort order for different import kinds:
// 1. Side-effect imports
// 2. Type-only imports
// 3. Namespace imports
// 4. Default imports
// 5. Named imports
// 6. ImportEqualsDeclarations
// 7. Require variable statements
// Go: ls/lsutil/organizeimports.go:331 importKindOrder consts
const IMPORT_KIND_ORDER_SIDE_EFFECT: i32 = 0;
const IMPORT_KIND_ORDER_TYPE_ONLY: i32 = 1;
const IMPORT_KIND_ORDER_NAMESPACE: i32 = 2;
const IMPORT_KIND_ORDER_DEFAULT: i32 = 3;
const IMPORT_KIND_ORDER_NAMED: i32 = 4;
const IMPORT_KIND_ORDER_IMPORT_EQUALS: i32 = 5;
const IMPORT_KIND_ORDER_REQUIRE: i32 = 6;
const IMPORT_KIND_ORDER_UNKNOWN: i32 = 7;

// Go: ls/lsutil/organizeimports.go:342 getImportKindOrder

// Go: ls/lsutil/organizeimports.go:370 CompareImportsOrRequireStatements
// CompareImportsOrRequireStatements compares two import or require statements.

// Go: ls/lsutil/organizeimports.go:377 compareImportOrExportSpecifiers

// Go: ls/lsutil/organizeimports.go:400 GetNamedImportSpecifierComparer
// GetNamedImportSpecifierComparer returns a comparer function for sorting import specifiers.
// PORT: the returned Go closure captures a copy of `preferences`; this one
// holds a clone.

// Go: ls/lsutil/organizeimports.go:414 GetImportSpecifierInsertionIndex
// GetImportSpecifierInsertionIndex returns the index at which to insert a new import specifier.
pub fn get_import_specifier_insertion_index(
    sorted_imports: &[Node],
    new_import: Node,
    comparer: &dyn Fn(Node, Node) -> i32,
) -> i32 {
    binary_search_unique_func(sorted_imports, |_mid, value| comparer(value, new_import)).0
}

// Go: ls/lsutil/organizeimports.go:421 GetImportDeclarationInsertIndex
// GetImportDeclarationInsertIndex returns the index at which to insert a new import declaration.
pub fn get_import_declaration_insert_index(
    sorted_imports: &[Node],
    new_import: Node,
    comparer: &dyn Fn(Node, Node) -> i32,
) -> i32 {
    binary_search_unique_func(sorted_imports, |_mid, value| comparer(value, new_import)).0
}

// Go: ls/lsutil/organizeimports.go:428 GetOrganizeImportsStringComparerWithDetection
// GetOrganizeImportsStringComparerWithDetection returns a string comparer based on detecting the order of import statements by the module specifier

// Go: ls/lsutil/organizeimports.go:433 getComparers

// Go: ls/lsutil/organizeimports.go:447 namedImportSortResult
struct NamedImportSortResult {
    named_import_comparer: Option<StringComparer>,
    type_order: OrganizeImportsTypeOrder,
    is_sorted: bool,
}

// Go: ls/lsutil/organizeimports.go:454 DetectNamedImportOrganizationBySort
// DetectNamedImportOrganizationBySort detects the order of named imports throughout the file by considering the named imports in each statement as a group
// PORT: the unexported Go `detectNamedImportOrganizationBySort` has the same
// snake name, so this exported one ends in `_exported` (PORTING "Names").

// Go: ls/lsutil/organizeimports.go:466 detectNamedImportOrganizationBySort
// PORT: Go `map[OrganizeImportsTypeOrder]T` reads give the zero value (0 or
// nil) for a missing key; `get(..).unwrap_or(0)` and `get(..).cloned()` do
// the same. Go `math.MaxInt` is `i32::MAX` here (PORTING `int` -> `i32`);
// diffs are counts far below either.

// Go: ls/lsutil/organizeimports.go:597 caseSensitivityDetectionResult
struct CaseSensitivityDetectionResult {
    comparer: Option<StringComparer>,
    is_sorted: bool,
}

// Go: ls/lsutil/organizeimports.go:603 DetectModuleSpecifierCaseBySort
// DetectModuleSpecifierCaseBySort detects the order of module specifiers based on import statements throughout the module/file

// Go: ls/lsutil/organizeimports.go:620 detectCaseSensitivityBySort
// PORT: Go `math.MaxInt` is `i32::MAX` here (PORTING `int` -> `i32`).
fn detect_case_sensitivity_by_sort(
    original_groups: &[Vec<String>],
    comparers_to_test: &[StringComparer],
) -> CaseSensitivityDetectionResult {
    let mut best_comparer: Option<StringComparer> = None;
    let mut best_diff = i32::MAX;

    for cur_comparer in comparers_to_test {
        let mut diff_of_current_comparer = 0;

        for list_to_sort in original_groups {
            if list_to_sort.len() <= 1 {
                continue;
            }
            let diff =
                measure_sortedness(list_to_sort, |a: &String, b: &String| cur_comparer(a, b));
            diff_of_current_comparer += diff;
        }

        if diff_of_current_comparer < best_diff {
            best_diff = diff_of_current_comparer;
            best_comparer = Some(cur_comparer.clone());
        }
    }

    if best_comparer.is_none() && !comparers_to_test.is_empty() {
        best_comparer = Some(comparers_to_test[0].clone());
    }

    CaseSensitivityDetectionResult {
        comparer: best_comparer,
        is_sorted: best_diff == 0,
    }
}

// Go: ls/lsutil/organizeimports.go:651 measureSortedness
// PORT: Go passes the elements by value; this passes references.
fn measure_sortedness<T>(arr: &[T], comparer: impl Fn(&T, &T) -> i32) -> i32 {
    let mut i = 0;
    for j in 0..arr.len().saturating_sub(1) {
        if comparer(&arr[j], &arr[j + 1]) > 0 {
            i += 1;
        }
    }
    i
}

// Go: ls/lsutil/organizeimports.go:662 GetNamedImportSpecifierComparerWithDetection
// GetNamedImportSpecifierComparerWithDetection returns a specifier comparer based on detecting the existing sort order within a single import statement

#[cfg(test)]
mod tests {
    use super::*;

    // Go: ls/lsutil/utilities_test.go:139 TestCompareOrganizeImportsNaturalStrings
    // PORT: in this file because it calls the unexported
    // `getOrganizeImportsPresetStringComparer`.
    #[test]
    fn test_compare_organize_imports_natural_strings() {
        let comparer =
            get_organize_imports_preset_string_comparer(OrganizeImportsSort::NATURAL_IGNORE_CASE);
        #[rustfmt::skip]
        let tests: &[(&str, &str, &str, i32)] = &[
            ("numeric runs sort by numeric value", "a2", "a100", -1),
            ("numeric runs with equal value use raw tie break", "a02", "a2", -1),
            ("accents are folded for primary comparison", "À", "B", -1),
            ("raw comparison breaks accent ties", "A", "À", -1),
            ("hyphen sorts before slash like Intl.Collator fallback", "app-init", "app/app", -1),
        ];
        for &(name, a, b, want) in tests {
            let got = comparer(a, b).signum();
            assert_eq!(
                got, want,
                "{name}: comparer({a:?}, {b:?}) = {got}, want sign {want}"
            );
        }
    }

    // Go `naturalCollationKey` (ls/lsutil/organizeimports.go:133) at pin N
    // runs on go1.27.1 and x/text v0.42.0 (Unicode 17.0.0): U+A7CB lowers to
    // U+0264, U+105C9 decomposes to U+105D2 U+0307 (Mn) and U+0897 is Mn.
    // The values are Go's; Unicode 15.0.0 kept all three.
    #[test]
    fn natural_collation_key_uses_unicode_17() {
        assert_eq!(natural_collation_key("\u{A7CB}a"), "\u{264}a");
        assert_eq!(natural_collation_key("\u{105C9}"), "\u{105D2}");
        assert_eq!(natural_collation_key("a\u{897}b"), "ab");
        let comparer =
            get_organize_imports_unicode_string_comparer(true, &UserPreferences::default());
        assert_eq!(comparer("./\u{A7CB}a", "./\u{264}b").signum(), -1);
    }
}
