use crate::prelude::*;
use std::sync::LazyLock;

// Port of checker/services.go (all except GetConstantValue at services.go:859,
// which is in emit_resolver_p2.rs).
//
// PORT: names follow `checker-api-tools/names_exports_services.tsv` column 5.
// An exported Go method whose unexported twin has the same snake name ends
// in `_exported`.
// PORT: Go package-level functions that read checker data
// (`runWithInferenceBlockedFromSourceNode`, `GetResolvedSignatureForSignatureHelp`,
// `runWithoutResolvedSignatureCaching`) are Checker methods. The others
// (`isExportSpecifierAlias`, `getPossibleSymbolReferenceNodes`,
// `getPossibleSymbolReferencePositions`, `isKnownGenericTypeName`) are free
// functions.
// PORT: a Go `[]*T` result is a `Vec<T>`.


// Go: checker/services.go:633 isExportSpecifierAlias

// Go: checker/services.go:646 getPossibleSymbolReferenceNodes

// Go: checker/services.go:655 getPossibleSymbolReferencePositions
// PORT: Go has two copies of this function with the same body (here and
// ls/findallreferences.go:1646). The port keeps one: the ls copy, which
// searches Go bytes and reads the bytes next to a match as Go bytes (see
// its PORT notes). Before, this copy read port bytes there, so a match
// next to an invalid byte unit tested a port byte (0x85 for Go's 0xC5).


// Go: checker/services.go:981 knownGenericTypeNames
static KNOWN_GENERIC_TYPE_NAMES: LazyLock<FxHashSet<&'static str>> = LazyLock::new(|| {
    [
        "Array",
        "ArrayLike",
        "ReadonlyArray",
        "Promise",
        "PromiseLike",
        "Iterable",
        "IterableIterator",
        "AsyncIterable",
        "Set",
        "WeakSet",
        "ReadonlySet",
        "Map",
        "WeakMap",
        "ReadonlyMap",
        "Partial",
        "Required",
        "Readonly",
        "Pick",
        "Omit",
        "NonNullable",
    ]
    .into_iter()
    .collect()
});

// Go: checker/services.go:1004 isKnownGenericTypeName
pub fn is_known_generic_type_name(name: &str) -> bool {
    KNOWN_GENERIC_TYPE_NAMES.contains(name)
}

