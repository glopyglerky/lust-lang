use crate::format::prelude::*;

use std::sync::Arc;

//
// Contexts
//

// Go: format/rulecontext.go:18 optionSelector
// PORT: Go selectors take `lsutil.FormatCodeSettings` by value; these read it
// by reference. They are plain fn pointers, so the static rules map stays
// `Send + Sync`.
// Go: format/rulecontext.go:19 anyOptionSelector

// Go: format/rulecontext.go:22 semicolonOption

// Go: format/rulecontext.go:26 insertSpaceAfterCommaDelimiterOption

// Go: format/rulecontext.go:30 insertSpaceAfterSemicolonInForStatementsOption

// Go: format/rulecontext.go:34 insertSpaceBeforeAndAfterBinaryOperatorsOption

// Go: format/rulecontext.go:38 insertSpaceAfterConstructorOption

// Go: format/rulecontext.go:42 insertSpaceAfterKeywordsInControlFlowStatementsOption

// Go: format/rulecontext.go:46 insertSpaceAfterFunctionKeywordForAnonymousFunctionsOption

// Go: format/rulecontext.go:50 insertSpaceAfterOpeningAndBeforeClosingNonemptyParenthesisOption

// Go: format/rulecontext.go:54 insertSpaceAfterOpeningAndBeforeClosingNonemptyBracketsOption

// Go: format/rulecontext.go:58 insertSpaceAfterOpeningAndBeforeClosingNonemptyBracesOption

// Go: format/rulecontext.go:62 insertSpaceAfterOpeningAndBeforeClosingEmptyBracesOption

// Go: format/rulecontext.go:66 insertSpaceAfterOpeningAndBeforeClosingTemplateStringBracesOption

// Go: format/rulecontext.go:70 insertSpaceAfterOpeningAndBeforeClosingJsxExpressionBracesOption

// Go: format/rulecontext.go:74 insertSpaceAfterTypeAssertionOption

// Go: format/rulecontext.go:78 insertSpaceBeforeFunctionParenthesisOption

// Go: format/rulecontext.go:82 placeOpenBraceOnNewLineForFunctionsOption

// Go: format/rulecontext.go:86 placeOpenBraceOnNewLineForControlBlocksOption

// Go: format/rulecontext.go:90 insertSpaceBeforeTypeAnnotationOption

// Go: format/rulecontext.go:94 indentMultiLineObjectLiteralBeginningOnBlankLineOption

// Go: format/rulecontext.go:98 indentSwitchCaseOption

// Go: format/rulecontext.go:102 optionEquals

// Go: format/rulecontext.go:108 isOptionEnabled

// Go: format/rulecontext.go:114 isOptionDisabled

// Go: format/rulecontext.go:120 isOptionDisabledOrUndefined

// Go: format/rulecontext.go:126 isOptionDisabledOrUndefinedOrTokensOnSameLine

// Go: format/rulecontext.go:132 isOptionEnabledOrUndefined

// Go: format/rulecontext.go:138 isForContext

// Go: format/rulecontext.go:142 isNotForContext

// Go: format/rulecontext.go:146 isBinaryOpContext

// Go: format/rulecontext.go:195 isNotBinaryOpContext

// Go: format/rulecontext.go:199 isNotTypeAnnotationContext

// Go: format/rulecontext.go:203 isTypeAnnotationContext

// Go: format/rulecontext.go:212 isOptionalPropertyContext

// Go: format/rulecontext.go:216 isNonOptionalPropertyContext

// Go: format/rulecontext.go:220 isConditionalOperatorContext

// Go: format/rulecontext.go:225 isSameLineTokenOrBeforeBlockContext

// Go: format/rulecontext.go:229 isBraceWrappedContext

// This check is done before an open brace in a control construct, a function, or a typescript block declaration
// Go: format/rulecontext.go:236 isBeforeMultilineBlockContext

// Go: format/rulecontext.go:240 isMultilineBlockContext

// Go: format/rulecontext.go:244 isSingleLineBlockContext

// Go: format/rulecontext.go:248 isBlockContext

// Go: format/rulecontext.go:252 isBeforeBlockContext

// IMPORTANT!!! This method must return true ONLY for nodes with open and close braces as immediate children
// Go: format/rulecontext.go:257 nodeIsBlockContext

// Go: format/rulecontext.go:274 isFunctionDeclContext

// Go: format/rulecontext.go:300 isNotFunctionDeclContext

// Go: format/rulecontext.go:304 isFunctionDeclarationOrFunctionExpressionContext

// Go: format/rulecontext.go:308 isTypeScriptDeclWithBlockContext

// Go: format/rulecontext.go:312 nodeIsTypeScriptDeclWithBlockContext

// Go: format/rulecontext.go:330 isAfterCodeBlockContext

// Go: format/rulecontext.go:349 isControlDeclContext

// Go: format/rulecontext.go:371 isObjectContext

// Go: format/rulecontext.go:375 isFunctionCallContext

// Go: format/rulecontext.go:379 isNewContext

// Go: format/rulecontext.go:383 isFunctionCallOrNewContext

// Go: format/rulecontext.go:387 isPreviousTokenNotComma

// Go: format/rulecontext.go:391 isNextTokenNotCloseBracket

// Go: format/rulecontext.go:395 isNextTokenNotCloseParen

// Go: format/rulecontext.go:399 isArrowFunctionContext

// Go: format/rulecontext.go:403 isImportTypeContext

// Go: format/rulecontext.go:407 isNonJsxSameLineTokenContext

// Go: format/rulecontext.go:411 isNonJsxTextContext

// Go: format/rulecontext.go:415 isNonJsxElementOrFragmentContext

// Go: format/rulecontext.go:419 isJsxExpressionContext

// Go: format/rulecontext.go:423 isNextTokenParentJsxAttribute

// Go: format/rulecontext.go:427 isJsxAttributeContext

// Go: format/rulecontext.go:431 isNextTokenParentNotJsxNamespacedName

// Go: format/rulecontext.go:435 isNextTokenParentJsxNamespacedName

// Go: format/rulecontext.go:439 isJsxSelfClosingElementContext

// Go: format/rulecontext.go:443 isNotBeforeBlockInFunctionDeclarationContext

// Go: format/rulecontext.go:447 isEndOfDecoratorContextOnSameLine

// Go: format/rulecontext.go:454 nodeIsInDecoratorContext

// Go: format/rulecontext.go:461 isStartOfVariableDeclarationList

// Go: format/rulecontext.go:466 isNotFormatOnEnter

// Go: format/rulecontext.go:470 isModuleDeclContext

// Go: format/rulecontext.go:474 isObjectTypeContext

// Go: format/rulecontext.go:478 isConstructorSignatureContext

// Go: format/rulecontext.go:482 isTypeArgumentOrParameterOrAssertion
// PORT: Go takes the TextRangeWithKind by value; this reads it by reference.

// Go: format/rulecontext.go:509 isTypeArgumentOrParameterOrAssertionContext

// Go: format/rulecontext.go:514 isTypeAssertionContext

// Go: format/rulecontext.go:518 isNonTypeAssertionContext

// Go: format/rulecontext.go:522 isVoidOpContext

// Go: format/rulecontext.go:526 isYieldOrYieldStarWithOperand

// Go: format/rulecontext.go:530 isNonNullAssertionContext

// Go: format/rulecontext.go:534 isNotStatementConditionContext

// Go: format/rulecontext.go:538 isStatementConditionContext

// Go: format/rulecontext.go:553 isSemicolonDeletionContext

// Go: format/rulecontext.go:621 isSemicolonInsertionContext

// Go: format/rulecontext.go:625 isNotPropertyAccessOnIntegerLiteral
