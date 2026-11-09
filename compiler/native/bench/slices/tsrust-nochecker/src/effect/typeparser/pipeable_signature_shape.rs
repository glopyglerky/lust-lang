//! Port of Effect-TS/tsgo `internal/typeparser/pipeable_signature_shape.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

use std::cell::Cell;

pub const PIPEABLE_SHAPE_MAX_DEPTH: i32 = 64;
pub const PIPEABLE_SHAPE_MAX_WORK: i32 = 4096;
pub const PIPEABLE_SHAPE_MAX_UNION_SEARCH: i32 = 128;



// Go: typeparser/pipeable_signature_shape.go rawSignature
// PORT: Go reads `signature.Target()` without a checker; here the signature
// arena is on the checker, so it takes `c`.

// Go: typeparser/pipeable_signature_shape.go pipeableSignatureShapesMatch


// Go: typeparser/pipeable_signature_shape.go comparePipeableSignatureTypes


// Go: typeparser/pipeable_signature_shape.go symbolIsOptional
// PORT: Go reads `symbol.Flags` without a checker; the symbol arena is on
// the checker here.


// Go: typeparser/pipeable_signature_shape.go sameOptionalSymbol

// Go: typeparser/pipeable_signature_shape.go noInferBaseType

// Go: typeparser/pipeable_signature_shape.go isNoInferAliasDeclaration
// Effect supported NoInfer before it became a TypeScript intrinsic with the
// standard tuple/indexed-access encoding. Recognize only those two exact
// declarations so an unrelated alias named NoInfer is not made transparent.

// Go: typeparser/pipeable_signature_shape.go isTypeParameterTypeNode

// Go: typeparser/pipeable_signature_shape.go isNumericLiteralTypeNode

