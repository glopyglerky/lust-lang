//! Port of typescript-go `internal/checker/checker.go` lines 23118-24045
//! (type arguments from nodes, tuple normalization, tuple/array predicates,
//! type alias references, declared types of symbols, enum member values,
//! and array/tuple type nodes).

use crate::jsnum::Number;
use crate::prelude::*;


// PORT: Go `TupleNormalizer` stores the checker in field `c`. Here the
// checker is passed to `normalize` and `add` instead, because the normalizer
// lives on the stack while the checker is mutably borrowed.
// Go: checker/checker.go:23811 TupleNormalizer
#[derive(Clone, Debug, Default)]
pub struct TupleNormalizer {
    pub types: Vec<TypeId>,
    pub infos: Vec<TupleElementInfo>,
    pub last_required_index: i32,
    pub first_rest_index: i32,
    pub last_optional_or_rest_index: i32,
}

impl TupleNormalizer {
    // Go: checker/checker.go:23820 TupleNormalizer.normalize

    // Go: checker/checker.go:23886 TupleNormalizer.add
}

// Return count of starting consecutive tuple elements of the given kind(s)
// Go: checker/checker.go:23901 getStartElementCount

// Return count of ending consecutive tuple elements of the given kind(s)
// Go: checker/checker.go:23911 getEndElementCount

// Go: checker/checker.go:23920 getTotalFixedElementCount


// Go: checker/checker.go:24152 getTypeReferenceName

