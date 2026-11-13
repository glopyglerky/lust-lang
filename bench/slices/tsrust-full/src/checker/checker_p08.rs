//! Port of Go `checker/checker.go` lines 6626-7527 (checker-08).
//! Iterator result types, alias checks, type alias checks, unused
//! identifier reporting, quick expression types and non-null checks.

use crate::prelude::*;


// Go: checker/checker.go:6898 isES2015OrLaterIterable
pub fn is_es2015_or_later_iterable(n: &str) -> bool {
    matches!(
        n,
        "Float32Array"
            | "Float64Array"
            | "Int16Array"
            | "Int32Array"
            | "Int8Array"
            | "NodeList"
            | "Uint16Array"
            | "Uint32Array"
            | "Uint8Array"
            | "Uint8ClampedArray"
    )
}


// Go: checker/checker.go:7439 isIdentifierThatStartsWithUnderscore

// Go: checker/checker.go:7443 importClauseFromImported

