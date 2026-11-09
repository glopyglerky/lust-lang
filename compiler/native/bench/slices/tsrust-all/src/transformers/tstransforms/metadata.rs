//! Port of `transformers/tstransforms/metadata.go`.

use super::type_serializer::{
    MetadataSerializerContext, 
};
use crate::prelude::*;
use crate::transformers::transformer::{
    
};

// Go: transformers/tstransforms/metadata.go:10 USE_NEW_TYPE_METADATA_FORMAT
const USE_NEW_TYPE_METADATA_FORMAT: bool = false;

// Go: transformers/tstransforms/metadata.go:12 MetadataTransformer

// Go: transformers/tstransforms/metadata.go:24 NewMetadataTransformer
// PORT: Go never returns nil here. The result is an `Option` so the
// constructor has the `TransformerFactory` shape.



