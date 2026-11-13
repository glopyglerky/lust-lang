//! Port of `checker/mapper.go`.
//!
//! Go `*TypeMapper` is a `MapperId` into `Checker::mappers`. The Go
//! `TypeMapperData` interface implementations become variants of the
//! `TypeMapper` enum. Go `m.Map(t)` is `self.mapper_map(m, t)`.

use crate::prelude::*;

// TypeMapperKind

// Go: checker/mapper.go:11 TypeMapperKind
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(i32)]
pub enum TypeMapperKind {
    #[default]
    Unknown = 0,
    Simple = 1,
    Array = 2,
    Merged = 3,
}

// TypeMapper




// 32-bit targets (wasm32) have smaller pointers and do not check this.




// TypeMapperBase

// Go: checker/mapper.go:102 TypeMapperBase
#[derive(Clone, Copy, Debug, Default)]
pub struct TypeMapperBase;

// SimpleTypeMapper

// Go: checker/mapper.go:112 SimpleTypeMapper
#[derive(Clone, Debug, Default)]
pub struct SimpleTypeMapper {
    pub source: TypeId,
    pub target: TypeId,
    /// Cached Go `isThisTypeParameter(m.source)`.
    pub maps_this_only: bool,
}

// ArrayTypeMapper

// Go: checker/mapper.go:143 ArrayTypeMapper
// PORT: the cached Go `MapsThisOnly`, `len(m.sources) == 1 &&
// isThisTypeParameter(m.sources[0])`, is the bool of `TypeMapper::Array`.

// ArrayToSingleTypeMapper

// Go: checker/mapper.go:176 ArrayToSingleTypeMapper
#[derive(Clone, Debug, Default)]
pub struct ArrayToSingleTypeMapper {
    pub sources: Vec<TypeId>,
    pub target: TypeId,
    /// Cached Go `len(m.sources) == 1 && isThisTypeParameter(m.sources[0])`.
    pub maps_this_only: bool,
}

// DeferredTypeMapper

// Go: checker/mapper.go:203 DeferredTypeMapper

// FunctionTypeMapper

// Go: checker/mapper.go:232 FunctionTypeMapper

// MergedTypeMapper

// Go: checker/mapper.go:250 MergedTypeMapper
#[derive(Clone, Copy, Debug, Default)]
pub struct MergedTypeMapper {
    pub m1: MapperId,
    pub m2: MapperId,
}

// CompositeTypeMapper

// Go: checker/mapper.go:274 CompositeTypeMapper
// PORT: the Go `c *Checker` field is dropped; `mapper_map` gets the checker.
#[derive(Clone, Copy, Debug, Default)]
pub struct CompositeTypeMapper {
    pub m1: MapperId,
    pub m2: MapperId,
}

// InferenceTypeMapper

// Go: checker/mapper.go:300 InferenceTypeMapper
// PORT: the Go `c *Checker` field is dropped; `mapper_map` gets the checker.
#[derive(Clone, Copy, Debug, Default)]
pub struct InferenceTypeMapper {
    pub n: InferenceContextId,
    pub fixing: bool,
}

