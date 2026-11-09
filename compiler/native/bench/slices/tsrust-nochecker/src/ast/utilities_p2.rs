//! Port of typescript-go `internal/ast/utilities.go` lines 906-1805.

use crate::astdata::NodeData;
use crate::prelude::*;

// Go: ast/utilities.go:906 SetImportsOfSourceFile
// PORT: skipped. Go documents it as "should never be called outside the
// parser". Parser data (`imports`) is fixed in `SourceFileInfo` when the
// program is installed, so there is nothing to set here.

// Go: ast/utilities.go:911 FindAncestor

// Go: ast/utilities.go:921 FindManyAncestors


// Go: ast/utilities.go:941 FindAncestorKind

// Go: ast/utilities.go:959 ToFindAncestorResult
pub fn to_find_ancestor_result(b: bool) -> FindAncestorResult {
    if b {
        return FindAncestorResult::FIND_ANCESTOR_TRUE;
    }
    FindAncestorResult::FIND_ANCESTOR_FALSE
}

// Go: ast/utilities.go:967 FindAncestorOrQuit

// Go: ast/utilities.go:980 IsNodeDescendantOf

// Go: ast/utilities.go:990 ModifierToFlag
pub fn modifier_to_flag(token: SyntaxKind) -> ModifierFlags {
    match token {
        SyntaxKind::StaticKeyword => ModifierFlags::STATIC,
        SyntaxKind::PublicKeyword => ModifierFlags::PUBLIC,
        SyntaxKind::ProtectedKeyword => ModifierFlags::PROTECTED,
        SyntaxKind::PrivateKeyword => ModifierFlags::PRIVATE,
        SyntaxKind::AbstractKeyword => ModifierFlags::ABSTRACT,
        SyntaxKind::AccessorKeyword => ModifierFlags::ACCESSOR,
        SyntaxKind::ExportKeyword => ModifierFlags::EXPORT,
        SyntaxKind::DeclareKeyword => ModifierFlags::AMBIENT,
        SyntaxKind::ConstKeyword => ModifierFlags::CONST,
        SyntaxKind::DefaultKeyword => ModifierFlags::DEFAULT,
        SyntaxKind::AsyncKeyword => ModifierFlags::ASYNC,
        SyntaxKind::ReadonlyKeyword => ModifierFlags::READONLY,
        SyntaxKind::OverrideKeyword => ModifierFlags::OVERRIDE,
        SyntaxKind::InKeyword => ModifierFlags::IN,
        SyntaxKind::OutKeyword => ModifierFlags::OUT,
        SyntaxKind::Decorator => ModifierFlags::DECORATOR,
        _ => ModifierFlags::NONE,
    }
}

// Go: ast/utilities.go:1028 ModifiersToFlags

// Go: ast/utilities.go:1036 HasSyntacticModifier


// Go: ast/utilities.go:1040 HasAccessorModifier

// Go: ast/utilities.go:1044 HasStaticModifier

// Go: ast/utilities.go:1048 IsStatic

// Go: ast/utilities.go:1053 CanHaveSymbol

// Go: ast/utilities.go:1072 CanHaveIllegalDecorators

// Go: ast/utilities.go:1088 CanHaveIllegalModifiers

// Go: ast/utilities.go:1100 CanHaveModifiers

// Go: ast/utilities.go:1133 CanHaveDecorators

// Go: ast/utilities.go:1147 IsFunctionOrModuleBlock

// Go: ast/utilities.go:1151 IsFunctionExpressionOrArrowFunction

// Go: ast/utilities.go:1157 ForEachReturnStatement

// Go: ast/utilities.go:1173 GetRootDeclaration

// Go: ast/utilities.go:1180 GetCombinedModifierFlags
// PERF: `get_root_declaration` is inlined here and each node kind is read
// once (it was read in `get_root_declaration` and again for the
// VariableDeclaration test, each read with the store bounds checks). The
// walk and the flag reads are the same as Go. A nil node fails the Go
// `node != nil` tests, so it returns early. The same holds for
// `get_combined_node_flags` and `get_combined_parser_flags`.

// Go: ast/utilities.go:1196 GetCombinedNodeFlags


// Go: ast/utilities.go:1213 IsVarAwaitUsing

// Go: ast/utilities.go:1218 IsVarUsing

// Go: ast/utilities.go:1223 GetJSDocDeprecatedTag

// Go: ast/utilities.go:1240 IsDeprecatedDeclaration

// Go: ast/utilities.go:1246 IsDeprecatedDeclarationWithCachedFlags

// Go: ast/utilities.go:1261 IsVarConst

// Go: ast/utilities.go:1266 IsVarConstLike

// Go: ast/utilities.go:1275 IsVarLet

// Go: ast/utilities.go:1279 IsImportMeta

// Go: ast/utilities.go:1286 WalkUpBindingElementsAndPatterns

// Go: ast/utilities.go:1294 IsSourceFileJS

// Go: ast/utilities.go:1298 IsInJSFile
// PERF: U4 (CH7). `JAVA_SCRIPT_FILE` is a parser bit (`Node::parser_flags`).

// Go: ast/utilities.go:1302 IsDeclaration

// Go: ast/utilities.go:1310 IsDeclarationName

// Go: ast/utilities.go:1315 IsDeclarationNameOrImportPropertyName

// Go: ast/utilities.go:1324 IsLiteralComputedPropertyDeclarationName

// Go: ast/utilities.go:1330 IsExternalModuleImportEqualsDeclaration

// Go: ast/utilities.go:1334 IsModuleOrEnumDeclaration

// Go: ast/utilities.go:1338 IsLiteralImportTypeNode

// Go: ast/utilities.go:1342 IsJsxTagName

// Go: ast/utilities.go:1351 IsImportOrExportSpecifier

// Go: ast/utilities.go:1355 IsVoidZero


// Go: ast/utilities.go:1359 IsExportsIdentifier
// PERF: compares name ids (`Node::text_is`), with no text load.

// Go: ast/utilities.go:1363 IsModuleIdentifier
// PERF: compares name ids (`Node::text_is`), with no text load.
// `is_module_exports_access_expression` calls this first, also for TS
// files (`check_testing_known_truthy_types`).


// Go: ast/utilities.go:1367 IsThisIdentifier
// PERF: U1 (a). Compares name ids (`Node::text_is`), with no text load.
// `is_this_in_type_query` calls this first.

// Go: ast/utilities.go:1371 IsThisParameter

// Go: ast/utilities.go:1375 IsBindableStaticAccessExpression

// Go: ast/utilities.go:1381 IsBindableStaticElementAccessExpression

// Go: ast/utilities.go:1388 IsPrototypeAccess

// Go: ast/utilities.go:1397 IsLiteralLikeElementAccess

// Go: ast/utilities.go:1401 IsBindableStaticNameExpression

// Go: ast/utilities.go:1407 GetElementOrPropertyAccessName

// Go: ast/utilities.go:1423 GetInitializerOfBinaryExpression

// Go: ast/utilities.go:1430 IsExpressionWithTypeArgumentsInClassExtendsClause

// Go: ast/utilities.go:1434 TryGetClassExtendingExpressionWithTypeArguments

// Go: ast/utilities.go:1445 TryGetClassImplementingOrExtendingHeritageClauseElement

// Go: ast/utilities.go:1453 GetNameOfDeclaration



// Go: ast/utilities.go:1467 GetNonAssignedNameOfDeclaration


// Go: ast/utilities.go:1492 GetAssignedName

// Go: ast/utilities.go:1547 GetAssignmentDeclarationKind



// Go: ast/utilities.go:1581 IsBindableObjectDefinePropertyCall

// Go: ast/utilities.go:1602 HasDynamicName


// Go: ast/utilities.go:1607 IsDynamicName

// Go: ast/utilities.go:1620 IsEntityNameExpression

// Go: ast/utilities.go:1624 IsEntityNameExpressionEx

// Go: ast/utilities.go:1630 IsPropertyAccessEntityNameExpression

// Go: ast/utilities.go:1634 isElementAccessEntityNameExpression

// Go: ast/utilities.go:1638 IsDottedName

// Go: ast/utilities.go:1648 HasSamePropertyAccessName

// Go: ast/utilities.go:1658 IsAmbientModule

// Go: ast/utilities.go:1662 IsAmbientModuleSymbolName

// Go: ast/utilities.go:1669 TryGetAmbientModuleNameFromSymbolName (ts#63931)
// Ambient module symbols are either of the form `"modulename"` or `InternalSymbolNamePrefix + "\"modulename\"pattern@nodeId"`;
// see `getDeclarationName`.
// PORT: `(string, bool)` returns `Option<&str>`. `s` is the port form of the
// Go name (see `INTERNAL_SYMBOL_NAME_PREFIX`); the marker index is only
// compared with 1, and the part before it is empty in both forms together.

// Go: ast/utilities.go:1686 IsExternalModule

// Go: ast/utilities.go:1690 IsExternalOrCommonJSModule

// Go: ast/utilities.go:1695 IsEffectiveExternalModule
// TODO(Go): Should we deprecate `IsExternalOrCommonJSModule` in favor of this function?

// Go: ast/utilities.go:1699 isCommonJSContainingModuleKind
pub fn is_common_js_containing_module_kind(kind: ModuleKind) -> bool {
    kind == ModuleKind::COMMON_JS || ModuleKind::NODE16 <= kind && kind <= ModuleKind::NODE_NEXT
}

// Go: ast/utilities.go:1703 IsExternalModuleIndicator

// Go: ast/utilities.go:1708 IsExportNamespaceAsDefaultDeclaration

// Go: ast/utilities.go:1716 IsGlobalScopeAugmentation

// Go: ast/utilities.go:1720 IsModuleAugmentationExternal

// Go: ast/utilities.go:1734 IsModuleWithStringLiteralName

// Go: ast/utilities.go:1738 GetContainingClass

// Go: ast/utilities.go:1742 GetExtendsHeritageClauseElements

// Go: ast/utilities.go:1746 GetImplementsHeritageClauseElements

// Go: ast/utilities.go:1750 GetHeritageElements

// Go: ast/utilities.go:1759 GetHeritageClauseElementName

// Go: ast/utilities.go:1766 IsNameOfHeritageClauseTypeReference

// Go: ast/utilities.go:1775 GetHeritageClause

// Go: ast/utilities.go:1787 getHeritageClauses

// Go: ast/utilities.go:1799 IsPartOfTypeQuery

// Go: ast/utilities.go:1813 IsPartOfParameterDeclaration

// Go: ast/utilities.go:1817 IsInTopLevelContext

// Go: ast/utilities.go:1829 GetThisContainer
