//! Go `internal/checker/utilities.go` lines 904-1844.

use crate::jsnum::PseudoBigInt;
use crate::prelude::*;
use std::cell::Cell;

// Go: checker/utilities.go:970 isValidNumberString

// Go: checker/utilities.go:978 isValidBigIntString

// Go: checker/utilities.go:1004 isValidESSymbolDeclaration

// Go: checker/utilities.go:1014 isVariableDeclarationInVariableStatement


// Go: checker/utilities.go:1029 isLateBoundName
// PORT: Go checks `name[0] == '\xfe' && name[1] == '@'`. The byte 0xFE is
// INTERNAL_SYMBOL_NAME_PREFIX in the port form (see `ast::misc`), so this
// checks for that prefix followed by the byte '@'.
pub fn is_late_bound_name(name: &str) -> bool {
    match name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        Some(rest) => !rest.is_empty() && rest.as_bytes()[0] == b'@',
        None => false,
    }
}


// Go: checker/utilities.go:1037 getContainingClassExcludingClassDecorators


// Go: checker/utilities.go:1060 isClassInstanceProperty

// Go: checker/utilities.go:1069 isThisInitializedObjectBindingExpression

// Go: checker/utilities.go:1075 isThisInitializedDeclaration

// Go: checker/utilities.go:1079 isInfinityOrNaNString
pub fn is_infinity_or_nan_string(name: &str) -> bool {
    name == "Infinity" || name == "-Infinity" || name == "NaN"
}


// Go: checker/utilities.go:1101 isInAmbientOrTypeNode
// PERF: U4 (CH7), as `Checker::is_in_ambient_or_type_node`.

// Go: checker/utilities.go:1107 isLiteralExpressionOfObject

// Go: checker/utilities.go:1116 canHaveFlowNode
// PORT: Go `node.FlowNodeData() != nil`. That is non-nil exactly for node
// kinds whose Go data struct embeds `FlowNodeBase` (directly or through
// StatementBase, IterationStatementBase or AccessorDeclarationBase, see
// ast/ast_generated.go), so this lists those kinds.

// Go: checker/utilities.go:1120 isNonNullAccess

// Go: checker/utilities.go:1124 getBindingElementPropertyName

// Go: checker/utilities.go:1128 isCallChain


// Go: checker/utilities.go:1136 isSuperCall
// PORT: identical to `ast.IsSuperCall`, which is already the public
// `is_super_call` in `crate::ast`. A second public item with the same name
// would make the prelude glob ambiguous, so this copy stays private and
// checker callers resolve to the identical ast function.

// Go: checker/utilities.go:1140 getMembersOfDeclaration

// Go: checker/utilities.go:1150 isInRightSideOfImportOrExportAssignment

// Go: checker/utilities.go:1159 isJsxIntrinsicTagName

// Go: checker/utilities.go:1163 getContainingObjectLiteral

// Go: checker/utilities.go:1174 isImportTypeQualifierPart

// Go: checker/utilities.go:1188 isInNameOfExpressionWithTypeArgumentsOrHeritageTypeReference


// Go: checker/utilities.go:1203 expressionResultIsUnused
// Indicates whether the result of an `Expression` will be unused.
// NOTE: This requires a node with a valid `parent` pointer.

// Go: checker/utilities.go:1228 pseudoBigIntToString
pub fn pseudo_big_int_to_string(value: &PseudoBigInt) -> String {
    value.to_string()
}

// Go: checker/utilities.go:1232 getSuperContainer
// PORT: identical to `ast.GetSuperContainer`, which is already the public
// `get_super_container` in `crate::ast`. This copy stays private to keep the
// prelude glob unambiguous; checker callers resolve to the identical ast
// function.

// Go: checker/utilities.go:1264 forEachYieldExpression

// Go: checker/utilities.go:1298 getEnclosingContainer


// Go: checker/utilities.go:1308 hasType


// Go: checker/utilities.go:1316 minAndMax
pub fn min_and_max<T: Copy>(slice: &[T], mut get_value: impl FnMut(T) -> i32) -> (i32, i32) {
    let mut min_value = 0;
    let mut max_value = 0;
    for (i, &element) in slice.iter().enumerate() {
        let value = get_value(element);
        if i == 0 {
            min_value = value;
            max_value = value;
        } else {
            min_value = min_value.min(value);
            max_value = max_value.max(value);
        }
    }
    (min_value, max_value)
}

// Go: checker/utilities.go:1331 FeatureMapEntry
#[derive(Clone, Debug)]
pub struct FeatureMapEntry {
    pub lib: &'static str,
    pub props: Vec<&'static str>,
}

// Go: checker/utilities.go:1336 getFeatureMap
// PORT: Go `sync.OnceValue` becomes a `OnceLock`. The map keys and entry
// order are copied from Go.
pub fn get_feature_map() -> &'static FxHashMap<&'static str, Vec<FeatureMapEntry>> {
    static FEATURE_MAP: std::sync::OnceLock<FxHashMap<&'static str, Vec<FeatureMapEntry>>> =
        std::sync::OnceLock::new();
    FEATURE_MAP.get_or_init(|| {
        fn entry(lib: &'static str, props: &[&'static str]) -> FeatureMapEntry {
            FeatureMapEntry {
                lib,
                props: props.to_vec(),
            }
        }
        let entries: Vec<(&'static str, Vec<FeatureMapEntry>)> = vec![
            (
                "Array",
                vec![
                    entry(
                        "es2015",
                        &[
                            "find",
                            "findIndex",
                            "fill",
                            "copyWithin",
                            "entries",
                            "keys",
                            "values",
                        ],
                    ),
                    entry("es2016", &["includes"]),
                    entry("es2019", &["flat", "flatMap"]),
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            ("Iterator", vec![entry("es2015", &[])]),
            ("IteratorConstructor", vec![entry("es2026", &["concat"])]),
            ("RawJSON", vec![entry("es2026", &[])]),
            ("JSON", vec![entry("es2026", &["isRawJSON", "rawJSON"])]),
            ("AsyncIterator", vec![entry("es2015", &[])]),
            (
                "ArrayBuffer",
                vec![entry(
                    "es2024",
                    &[
                        "maxByteLength",
                        "resizable",
                        "resize",
                        "detached",
                        "transfer",
                        "transferToFixedLength",
                    ],
                )],
            ),
            (
                "Atomics",
                vec![
                    entry(
                        "es2017",
                        &[
                            "add",
                            "and",
                            "compareExchange",
                            "exchange",
                            "isLockFree",
                            "load",
                            "or",
                            "store",
                            "sub",
                            "wait",
                            "notify",
                            "xor",
                        ],
                    ),
                    entry("es2024", &["waitAsync"]),
                ],
            ),
            (
                "SharedArrayBuffer",
                vec![
                    entry("es2017", &["byteLength", "slice"]),
                    entry("es2024", &["growable", "maxByteLength", "grow"]),
                ],
            ),
            ("AsyncIterable", vec![entry("es2018", &[])]),
            ("AsyncIterableIterator", vec![entry("es2018", &[])]),
            ("AsyncGenerator", vec![entry("es2018", &[])]),
            ("AsyncGeneratorFunction", vec![entry("es2018", &[])]),
            (
                "RegExp",
                vec![
                    entry("es2015", &["flags", "sticky", "unicode"]),
                    entry("es2018", &["dotAll"]),
                    entry("es2024", &["unicodeSets"]),
                ],
            ),
            ("RegExpConstructor", vec![entry("es2025", &["escape"])]),
            (
                "Reflect",
                vec![entry(
                    "es2015",
                    &[
                        "apply",
                        "construct",
                        "defineProperty",
                        "deleteProperty",
                        "get",
                        "getOwnPropertyDescriptor",
                        "getPrototypeOf",
                        "has",
                        "isExtensible",
                        "ownKeys",
                        "preventExtensions",
                        "set",
                        "setPrototypeOf",
                    ],
                )],
            ),
            (
                "ArrayConstructor",
                vec![
                    entry("es2015", &["from", "of"]),
                    entry("es2026", &["fromAsync"]),
                ],
            ),
            (
                "ObjectConstructor",
                vec![
                    entry(
                        "es2015",
                        &[
                            "assign",
                            "getOwnPropertySymbols",
                            "keys",
                            "is",
                            "setPrototypeOf",
                        ],
                    ),
                    entry(
                        "es2017",
                        &["values", "entries", "getOwnPropertyDescriptors"],
                    ),
                    entry("es2019", &["fromEntries"]),
                    entry("es2022", &["hasOwn"]),
                    entry("es2024", &["groupBy"]),
                ],
            ),
            (
                "NumberConstructor",
                vec![entry(
                    "es2015",
                    &[
                        "isFinite",
                        "isInteger",
                        "isNaN",
                        "isSafeInteger",
                        "parseFloat",
                        "parseInt",
                    ],
                )],
            ),
            (
                "Math",
                vec![
                    entry(
                        "es2015",
                        &[
                            "clz32", "imul", "sign", "log10", "log2", "log1p", "expm1", "cosh",
                            "sinh", "tanh", "acosh", "asinh", "atanh", "hypot", "trunc", "fround",
                            "cbrt",
                        ],
                    ),
                    entry("es2025", &["f16round"]),
                    entry("es2026", &["sumPrecise"]),
                ],
            ),
            (
                "Map",
                vec![
                    entry("es2015", &["entries", "keys", "values"]),
                    entry("es2026", &["getOrInsert", "getOrInsertComputed"]),
                ],
            ),
            ("MapConstructor", vec![entry("es2024", &["groupBy"])]),
            (
                "Set",
                vec![
                    entry("es2015", &["entries", "keys", "values"]),
                    entry(
                        "es2025",
                        &[
                            "union",
                            "intersection",
                            "difference",
                            "symmetricDifference",
                            "isSubsetOf",
                            "isSupersetOf",
                            "isDisjointFrom",
                        ],
                    ),
                ],
            ),
            (
                "PromiseConstructor",
                vec![
                    entry("es2015", &["all", "race", "reject", "resolve"]),
                    entry("es2020", &["allSettled"]),
                    entry("es2021", &["any"]),
                    entry("es2024", &["withResolvers"]),
                    entry("es2025", &["try"]),
                ],
            ),
            (
                "Symbol",
                vec![
                    entry("es2015", &["for", "keyFor"]),
                    entry("es2019", &["description"]),
                ],
            ),
            (
                "WeakMap",
                vec![
                    entry("es2015", &[]),
                    entry("es2026", &["getOrInsert", "getOrInsertComputed"]),
                ],
            ),
            ("WeakSet", vec![entry("es2015", &[])]),
            (
                "String",
                vec![
                    entry(
                        "es2015",
                        &[
                            "codePointAt",
                            "includes",
                            "endsWith",
                            "normalize",
                            "repeat",
                            "startsWith",
                            "anchor",
                            "big",
                            "blink",
                            "bold",
                            "fixed",
                            "fontcolor",
                            "fontsize",
                            "italics",
                            "link",
                            "small",
                            "strike",
                            "sub",
                            "sup",
                        ],
                    ),
                    entry("es2017", &["padStart", "padEnd"]),
                    entry("es2019", &["trimStart", "trimEnd", "trimLeft", "trimRight"]),
                    entry("es2020", &["matchAll"]),
                    entry("es2021", &["replaceAll"]),
                    entry("es2022", &["at"]),
                    entry("es2024", &["isWellFormed", "toWellFormed"]),
                ],
            ),
            (
                "StringConstructor",
                vec![entry("es2015", &["fromCodePoint", "raw"])],
            ),
            ("DateTimeFormat", vec![entry("es2017", &["formatToParts"])]),
            (
                "Promise",
                vec![entry("es2015", &[]), entry("es2018", &["finally"])],
            ),
            ("RegExpMatchArray", vec![entry("es2018", &["groups"])]),
            ("RegExpExecArray", vec![entry("es2018", &["groups"])]),
            (
                "Intl",
                vec![
                    entry("es2018", &["PluralRules"]),
                    entry("es2020", &["RelativeTimeFormat", "Locale", "DisplayNames"]),
                    entry("es2021", &["ListFormat", "DateTimeFormat"]),
                    entry("es2022", &["Segmenter"]),
                    entry("es2025", &["DurationFormat"]),
                ],
            ),
            ("NumberFormat", vec![entry("es2018", &["formatToParts"])]),
            (
                "SymbolConstructor",
                vec![
                    entry("es2020", &["matchAll"]),
                    entry("esnext", &["metadata", "dispose", "asyncDispose"]),
                ],
            ),
            (
                "DataView",
                vec![
                    entry(
                        "es2020",
                        &["setBigInt64", "setBigUint64", "getBigInt64", "getBigUint64"],
                    ),
                    entry("es2025", &["setFloat16", "getFloat16"]),
                ],
            ),
            ("BigInt", vec![entry("es2020", &[])]),
            (
                "RelativeTimeFormat",
                vec![entry(
                    "es2020",
                    &["format", "formatToParts", "resolvedOptions"],
                )],
            ),
            (
                "Int8Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Uint8Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                    entry(
                        "es2026",
                        &["toBase64", "setFromBase64", "toHex", "setFromHex"],
                    ),
                ],
            ),
            (
                "Uint8ClampedArray",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Int16Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Uint16Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Int32Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Uint32Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            ("Float16Array", vec![entry("es2025", &[])]),
            (
                "Float32Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "Float64Array",
                vec![
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "BigInt64Array",
                vec![
                    entry("es2020", &[]),
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            (
                "BigUint64Array",
                vec![
                    entry("es2020", &[]),
                    entry("es2022", &["at"]),
                    entry(
                        "es2023",
                        &[
                            "findLastIndex",
                            "findLast",
                            "toReversed",
                            "toSorted",
                            "toSpliced",
                            "with",
                        ],
                    ),
                ],
            ),
            ("Error", vec![entry("es2022", &["cause"])]),
            ("ErrorConstructor", vec![entry("es2026", &["isError"])]),
            (
                "Uint8ArrayConstructor",
                vec![entry("es2026", &["fromBase64", "fromHex"])],
            ),
            ("DisposableStack", vec![entry("esnext", &[])]),
            ("AsyncDisposableStack", vec![entry("esnext", &[])]),
            ("Date", vec![entry("esnext", &["toTemporalInstant"])]),
        ];
        entries.into_iter().collect()
    })
}

// Go: checker/utilities.go:1609 rangeOfTypeParameters

// Go: checker/utilities.go:1613 tryGetPropertyAccessOrIdentifierToString
// PORT: Go `entityNameToString` here is the checker package function
// (utilities.go), which is `ast.EntityNameToString(name, scanner.GetTextOfNode)`.
// The ast version is called directly with the same text getter.


// Go: checker/utilities.go:1657 getAnyImportSyntax

// Go: checker/utilities.go:1677 isReservedMemberName
// A reserved member name consists of the byte 0xFE (which is an invalid UTF-8 encoding) followed by one or more
// characters where the first character is not '@' or '#'. The '@' character indicates that the name is denoted by
// a well known ES Symbol instance and the '#' character indicates that the name is a PrivateIdentifier.
// PORT: the byte 0xFE is INTERNAL_SYMBOL_NAME_PREFIX in the port form (see
// `ast::misc`), so "name[0]" is that prefix and "name[1]" is the first byte
// after it.
pub fn is_reserved_member_name(name: &str) -> bool {
    match name.strip_prefix(INTERNAL_SYMBOL_NAME_PREFIX) {
        Some(rest) => !rest.is_empty() && rest.as_bytes()[0] != b'@' && rest.as_bytes()[0] != b'#',
        None => false,
    }
}

// Go: checker/utilities.go:1681 introducesArgumentsExoticObject


// Go: checker/utilities.go:1749 ValueToString
// PORT: Go takes `any` and panics on other types. `LiteralValue` is closed
// over the four handled types, so the panic case cannot happen.

// Go: checker/utilities.go:1763 nodeStartsNewLexicalEnvironment


// Go: checker/utilities.go:1830 DiagnosticDetails
// DiagnosticDetails holds a resolved diagnostic message and its arguments,
// used for sharing diagnostic chain computation between the checker and incremental builder.
// PORT: Go `Args []any` holds only strings here, so the port uses
// `Vec<String>`, the argument type of `new_diagnostic_for_node`.

// Go: checker/utilities.go:1839 CreateModuleNotFoundChain
// CreateModuleNotFoundChain computes the diagnostic message and arguments for a module-not-found
// error chain entry. This is shared between the checker (initial diagnostic creation) and the
// incremental builder (repopulation of cached diagnostics).
// Mirrors createModuleNotFoundChain in the TypeScript compiler's utilities.ts.
// PORT: Go `program Program` is the installed `&'static GoProgram`. Its
// methods are the program.rs free functions, so the parameter is unused.
// `program.GetPackagesMap()` is `program::get_packages_map()`.


// Go: checker/utilities.go:1875 CreateModeMismatchDetails
// CreateModeMismatchDetails computes the diagnostic message and arguments for a mode-mismatch
// error chain entry. This is shared between the checker (initial diagnostic creation) and the
// incremental builder (repopulation of cached diagnostics).
// Mirrors createModeMismatchDetails in the TypeScript compiler's utilities.ts.
// PORT: `program` is unused for the same reason as in
// `create_module_not_found_chain`.

// Go: checker/utilities.go:1906 walkUpOuterExpressions

// Go: checker/utilities.go:1914 GetSetAccessorValueParameter

// PORT: private ports of the Go `tspath` extension helpers and the
// `module.MangleScopedPackageName` / `module.GetTypesPackageName` helpers
// used above. No shared Rust port of them exists in this crate.
mod tspath_up2 {
    // Go: tspath/extension.go:9
    pub const EXTENSION_TS: &str = ".ts";
    pub const EXTENSION_TSX: &str = ".tsx";
    pub const EXTENSION_DTS: &str = ".d.ts";
    pub const EXTENSION_JS: &str = ".js";
    pub const EXTENSION_JSX: &str = ".jsx";
    pub const EXTENSION_JSON: &str = ".json";
    pub const EXTENSION_MJS: &str = ".mjs";
    pub const EXTENSION_MTS: &str = ".mts";
    pub const EXTENSION_DMTS: &str = ".d.mts";
    pub const EXTENSION_CJS: &str = ".cjs";
    pub const EXTENSION_CTS: &str = ".cts";
    pub const EXTENSION_DCTS: &str = ".d.cts";

    // Go: tspath/extension.go:43 extensionsToRemove
    const EXTENSIONS_TO_REMOVE: [&str; 12] = [
        EXTENSION_DTS,
        EXTENSION_DMTS,
        EXTENSION_DCTS,
        EXTENSION_MJS,
        EXTENSION_MTS,
        EXTENSION_CJS,
        EXTENSION_CTS,
        EXTENSION_TS,
        EXTENSION_JS,
        EXTENSION_TSX,
        EXTENSION_JSX,
        EXTENSION_JSON,
    ];

    // Go: tspath/path.go:1095 FileExtensionIs
    fn file_extension_is(path: &str, extension: &str) -> bool {
        path.len() > extension.len() && path.ends_with(extension)
    }

    // Go: tspath/extension.go:66 TryGetExtensionFromPath
    pub fn try_get_extension_from_path(p: &str) -> &'static str {
        for ext in EXTENSIONS_TO_REMOVE {
            if file_extension_is(p, ext) {
                return ext;
            }
        }
        ""
    }

    // Go: module/util.go:58 MangleScopedPackageName

    // Go: module/util.go:77 GetTypesPackageName
}
