//! Port of `transformers/tstransforms/legacydecorators.go`.

use crate::prelude::*;
use crate::transformers::transformer::{
    TransformReferenceResolver, 
};
use crate::transformers::utilities::{
    
};

// Go: transformers/tstransforms/legacydecorators.go:12 LegacyDecoratorsTransformer

// Go: transformers/tstransforms/legacydecorators.go:25 NewLegacyDecoratorsTransformer
// PORT: Go never returns nil here. The result is an `Option` so the
// constructor has the `TransformerFactory` shape.




// Go: transformers/tstransforms/legacydecorators.go:94 elideNodes

// Go: transformers/tstransforms/legacydecorators.go:106 elideModifiers


// Go: transformers/tstransforms/legacydecorators.go:614 isClassStaticBlockDeclarationOrStaticProperty

// Go: transformers/tstransforms/legacydecorators.go:618 isNotExportOrDefaultOrDecorator

// Go: transformers/tstransforms/legacydecorators.go:622 decoratorContainsPrivateIdentifierInExpression

// Go: transformers/tstransforms/legacydecorators.go:626 parameterDecoratorsContainPrivateIdentifierInExpression

// Go: transformers/tstransforms/legacydecorators.go:630 hasClassElementWithDecoratorContainingPrivateIdentifierInExpression

// Go: transformers/tstransforms/legacydecorators.go:652 allDecorators
pub(super) struct AllDecorators {
    pub(super) decorators: Vec<Node>,
    pub(super) parameters: Vec<Vec<Node>>,
}

// Go: transformers/tstransforms/legacydecorators.go:665 getAllDecoratorsOfClass

// Go: transformers/tstransforms/legacydecorators.go:685 getAllDecoratorsOfClassElement

// Go: transformers/tstransforms/legacydecorators.go:707 getAllDecoratorsOfAccessors

// Go: transformers/tstransforms/legacydecorators.go:739 getAllDecoratorsOfProperty

// Go: transformers/tstransforms/legacydecorators.go:747 getAllDecoratorsOfMethod

// Go: transformers/tstransforms/legacydecorators.go:768 getDecoratorsOfParameters

// Go: transformers/tstransforms/legacydecorators.go:837 isDecoratedClassElement

// Go: transformers/tstransforms/legacydecorators.go:849 getDecoratedClassElements
