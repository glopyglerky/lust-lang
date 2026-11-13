//! Port of `transformers/tstransforms/typeserializer.go`.

use crate::prelude::*;

// Go: transformers/tstransforms/typeserializer.go:11 metadataSerializer
// PORT: Go keeps both the factory and the emit context; the factory is
// `ec.factory()` here.

// Go: transformers/tstransforms/typeserializer.go:20 metadataSerializerContext
#[derive(Clone, Copy, Default)]
pub(super) struct MetadataSerializerContext {
    pub(super) current_lexical_scope: Node,
    pub(super) current_name_scope: Node,
    pub(super) serializing_conditional_type_branch: bool,
}

// Go: transformers/tstransforms/typeserializer.go:26 newMetadataSerializer


// Go: transformers/tstransforms/typeserializer.go:55 GetSetAccessorValueParameter

// Go: transformers/tstransforms/typeserializer.go:70 getSetAccessorTypeAnnotationNode

// Go: transformers/tstransforms/typeserializer.go:78 getAccessorTypeNode

// Go: transformers/tstransforms/typeserializer.go:137 getParametersOfDecoratedDeclaration
