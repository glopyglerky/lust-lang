//! Port of Go `transformers/estransforms/namedevaluation.go`.

use crate::prelude::*;

// Go: transformers/estransforms/namedevaluation.go:16 isClassNamedEvaluationHelperBlock

// Go: transformers/estransforms/namedevaluation.go:38 classHasExplicitlyAssignedName

// Go: transformers/estransforms/namedevaluation.go:54 classHasDeclaredOrExplicitlyAssignedName

/// Go `func(*anonymousFunctionDefinition) bool`.
pub(crate) type AnonymousFunctionDefinitionCallback<'a> = &'a mut dyn FnMut(Node) -> bool;

// Go: transformers/estransforms/namedevaluation.go:63 isAnonymousFunctionDefinition

// Go: transformers/estransforms/namedevaluation.go:85 isNamedEvaluation

// Go: transformers/estransforms/namedevaluation.go:89 isNamedEvaluationAnd

// Go: transformers/estransforms/namedevaluation.go:109 getAssignedNameOfIdentifier

// Go: transformers/estransforms/namedevaluation.go:118 getAssignedNameOfPropertyName

// Go: transformers/estransforms/namedevaluation.go:153 createClassNamedEvaluationHelperBlock

// Go: transformers/estransforms/namedevaluation.go:176 injectClassNamedEvaluationHelperBlockIfMissing

// Go: transformers/estransforms/namedevaluation.go:250 finishTransformNamedEvaluation


// Go: transformers/estransforms/namedevaluation.go:273 transformNamedEvaluationOfPropertyAssignment

// Go: transformers/estransforms/namedevaluation.go:287 transformNamedEvaluationOfShorthandAssignmentProperty

// Go: transformers/estransforms/namedevaluation.go:315 transformNamedEvaluationOfVariableDeclaration

// Go: transformers/estransforms/namedevaluation.go:347 transformNamedEvaluationOfParameterDeclaration

// Go: transformers/estransforms/namedevaluation.go:383 transformNamedEvaluationOfBindingElement

// Go: transformers/estransforms/namedevaluation.go:417 transformNamedEvaluationOfPropertyDeclaration

// Go: transformers/estransforms/namedevaluation.go:438 transformNamedEvaluationOfAssignmentExpression

// Go: transformers/estransforms/namedevaluation.go:483 transformNamedEvaluationOfExportAssignment

// Go: transformers/estransforms/namedevaluation.go:513 transformNamedEvaluation
