//! Port of Effect-TS/tsgo `internal/typeparser/pipeable_signature_shape.go`.

use crate::effect::typeparser::*;
use crate::prelude::*;

use std::cell::Cell;

pub const PIPEABLE_SHAPE_MAX_DEPTH: i32 = 64;
pub const PIPEABLE_SHAPE_MAX_WORK: i32 = 4096;
pub const PIPEABLE_SHAPE_MAX_UNION_SEARCH: i32 = 128;

/// Go `pipeableSignatureShapeCacheKey`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipeableSignatureShapeCacheKey {
    pub data_first: SignatureId,
    pub candidate: SignatureId,
    pub subject_index: i32,
}

/// Go `pipeableTypePair`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PipeableTypePair {
    pub left: TypeId,
    pub right: TypeId,
}

// Go: typeparser/pipeable_signature_shape.go rawSignature
// PORT: Go reads `signature.Target()` without a checker; here the signature
// arena is on the checker, so it takes `c`.

// Go: typeparser/pipeable_signature_shape.go pipeableSignatureShapesMatch

/// Go `signatureTypeMatcher`.
/// signatureTypeMatcher zips existing, uninstantiated compiler type graphs. It
/// never asks the checker to relate two signatures and never creates a mapper.
/// Generic binders are paired by structural occurrence rather than declaration
/// order, with maps in both directions to preserve binder identity.
// PORT: Go keeps `checker *checker.Checker` in the struct. A clone of the
// matcher (compareUnionAt) would need a second `&mut Checker`, so every
// method takes `c` instead. Go `work *int` and `unionSearch *int` are shared
// by all clones: `Rc<Cell<i32>>`. Go ranges `leftToRight` in map order (in
// validateBinders); the port uses insertion order.
#[derive(Clone)]
pub struct SignatureTypeMatcher {
    pub left_to_right: IndexMap<TypeId, TypeId>,
    pub right_to_left: FxHashMap<TypeId, TypeId>,
    pub left_binders: FxHashSet<TypeId>,
    pub right_binders: FxHashSet<TypeId>,
    pub active: FxHashSet<PipeableTypePair>,
    pub work: Rc<Cell<i32>>,
    pub union_search: Rc<Cell<i32>>,
}

// Go: typeparser/pipeable_signature_shape.go comparePipeableSignatureTypes

impl SignatureTypeMatcher {
    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.registerBinders

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareParameter
}

// Go: typeparser/pipeable_signature_shape.go symbolIsOptional
// PORT: Go reads `symbol.Flags` without a checker; the symbol arena is on
// the checker here.

impl SignatureTypeMatcher {
    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareType

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareTypeParameter

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareObject

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareMappedType

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareAnonymousObject

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareSignature

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareTypeList

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareUnion

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareUnionAt

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.clone
    // PORT: named `clone_matcher` so it does not shadow `Clone::clone`. Go
    // copies the struct (sharing `work` and `unionSearch`) and clones the
    // five maps, which is what the derived `Clone` does.
    pub fn clone_matcher(&self) -> SignatureTypeMatcher {
        self.clone()
    }

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.commit
    pub fn commit(&mut self, branch: SignatureTypeMatcher) {
        self.left_to_right = branch.left_to_right;
        self.right_to_left = branch.right_to_left;
        self.left_binders = branch.left_binders;
        self.right_binders = branch.right_binders;
    }

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.validateBinders

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareOptionalType

    // Go: typeparser/pipeable_signature_shape.go signatureTypeMatcher.compareOptionalTypeAt
}

// Go: typeparser/pipeable_signature_shape.go sameOptionalSymbol

// Go: typeparser/pipeable_signature_shape.go noInferBaseType

// Go: typeparser/pipeable_signature_shape.go isNoInferAliasDeclaration
// Effect supported NoInfer before it became a TypeScript intrinsic with the
// standard tuple/indexed-access encoding. Recognize only those two exact
// declarations so an unrelated alias named NoInfer is not made transparent.

// Go: typeparser/pipeable_signature_shape.go isTypeParameterTypeNode

// Go: typeparser/pipeable_signature_shape.go isNumericLiteralTypeNode

