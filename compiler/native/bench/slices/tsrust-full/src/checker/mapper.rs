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

// ArrayTypeMapper

// Go: checker/mapper.go:143 ArrayTypeMapper
// PORT: the cached Go `MapsThisOnly`, `len(m.sources) == 1 &&
// isThisTypeParameter(m.sources[0])`, is the bool of `TypeMapper::Array`.

// ArrayToSingleTypeMapper

// Go: checker/mapper.go:176 ArrayToSingleTypeMapper

// DeferredTypeMapper

// Go: checker/mapper.go:203 DeferredTypeMapper

// FunctionTypeMapper

// Go: checker/mapper.go:232 FunctionTypeMapper

// MergedTypeMapper

// Go: checker/mapper.go:250 MergedTypeMapper

// CompositeTypeMapper

// Go: checker/mapper.go:274 CompositeTypeMapper
// PORT: the Go `c *Checker` field is dropped; `mapper_map` gets the checker.

// InferenceTypeMapper

// Go: checker/mapper.go:300 InferenceTypeMapper
// PORT: the Go `c *Checker` field is dropped; `mapper_map` gets the checker.

