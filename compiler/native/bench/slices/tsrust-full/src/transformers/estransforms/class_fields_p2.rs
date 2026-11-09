//! Port of Go `transformers/estransforms/classfields.go` lines 1592 to the
//! end: `createPrivateIdentifierAssignment` through
//! `createAccessorPropertySetRedirector`. The first part and the free helpers
//! `flattenCommaList`, `findComputedPropertyNameCacheAssignment` and
//! `expandPreOrPostfixIncrementOrDecrementExpression` are in
//! `class_fields.rs`.

use super::class_fields::{
    CLASS_FACTS_CLASS_WAS_DECORATED, CLASS_FACTS_NEEDS_CLASS_CONSTRUCTOR_REFERENCE,
    CLASS_FACTS_NEEDS_CLASS_SUPER_REFERENCE,
    CLASS_FACTS_NEEDS_SUBSTITUTION_FOR_THIS_IN_CLASS_STATIC_FIELD, CLASS_FACTS_NONE,
    CLASS_FACTS_WILL_HOIST_INITIALIZERS_TO_CONSTRUCTOR, ClassFacts, 
    ClassLexicalEnv, ClassLexicalEnvironment, ClassLexicalEnvironmentRef, PrivateEnvironment,
    PrivateEnvironmentRef, PrivateIdentifierInfo, PrivateIdentifierInfoRef,
    
    reserved_in_nested_scopes,
};
use super::named_evaluation::{
    
    
};
use crate::prelude::*;
use crate::printer::{AutoGenerateOptions, EmitFlags, GeneratedIdentifierFlags};
use crate::transformers::utilities::{
    get_non_assignment_operator_for_compound_assignment,
    
    
};


// Go: transformers/estransforms/classfields.go:3365 createPrivateStaticFieldInitializer

// Go: transformers/estransforms/classfields.go:3380 createPrivateInstanceFieldInitializer

// Go: transformers/estransforms/classfields.go:3387 createPrivateInstanceMethodInitializer

// Go: transformers/estransforms/classfields.go:3423 classHasClassThisAssignment

// Go: transformers/estransforms/classfields.go:3432 isNonStaticMethodOrAccessorWithPrivateName

// Go: transformers/estransforms/classfields.go:3438 createMemberAccessForPropertyName

// Go: transformers/estransforms/classfields.go:3483 shouldBeCapturedInTempVariable
