//! Port of Go `printer/factory.go`: the printer `NodeFactory`, which wraps
//! `ast.NodeFactory` with `EmitContext` hooks and adds emit-time helpers.

use crate::prelude::*;

use super::emit_context::{
    AutoGenerateInfo, AutoGenerateOptions, next_auto_generate_id,
};
use super::helpers::*;
use super::types::{EmitFlags, GeneratedIdentifierFlags};
use super::utilities::format_generated_name;
use std::rc::Weak;

// Go: printer/factory.go:13 NodeFactory


impl NodeFactory {
    // Go: printer/factory.go:18 NewNodeFactory
    // PORT: an associated fn, so the name does not clash with Go
    // `ast.NewNodeFactory`. `new_emit_context` (U3) calls it. The hooks hold a
    // `Weak` to the context, so they add no second `Rc` cycle; Go closes over
    // the pointer.


    // Go: ast/ast.go:99 AsNodeFactory (promoted from the embedded ast.NodeFactory)

    // Go: printer/factory.go:29 newGeneratedIdentifier

    // Go: printer/factory.go:62 NewTempVariable

    // Go: printer/factory.go:69 NewTempVariableEx

    // Go: printer/factory.go:74 NewLoopVariable

    // Go: printer/factory.go:79 NewLoopVariableEx

    // Go: printer/factory.go:84 NewUniqueName

    // Go: printer/factory.go:89 NewUniqueNameEx

    // Go: printer/factory.go:94 NewGeneratedNameForNode

    // Go: printer/factory.go:99 NewGeneratedNameForNodeEx

    // Go: printer/factory.go:107 newGeneratedPrivateIdentifier

    // Go: printer/factory.go:140 NewUniquePrivateName

    // Go: printer/factory.go:145 NewUniquePrivateNameEx

    // Go: printer/factory.go:150 NewGeneratedPrivateNameForNode

    // Go: printer/factory.go:155 NewGeneratedPrivateNameForNodeEx

    // Go: printer/factory.go:165 NewStringLiteralFromNode

    //
    // Common Tokens
    //

    // Go: printer/factory.go:193 NewThisExpression

    // Go: printer/factory.go:197 NewTrueExpression

    // Go: printer/factory.go:201 NewFalseExpression

    //
    // Common Operators
    //

    // Go: printer/factory.go:209 NewCommaExpression

    // Go: printer/factory.go:213 NewAssignmentExpression

    // Go: printer/factory.go:217 NewLogicalORExpression

    // Go: printer/factory.go:221 NewLogicalANDExpression

    // func (f *NodeFactory) NewLogicalANDExpression(left *ast.Expression, right *ast.Expression) *ast.Expression
    // func (f *NodeFactory) NewBitwiseORExpression(left *ast.Expression, right *ast.Expression) *ast.Expression
    // func (f *NodeFactory) NewBitwiseXORExpression(left *ast.Expression, right *ast.Expression) *ast.Expression
    // func (f *NodeFactory) NewBitwiseANDExpression(left *ast.Expression, right *ast.Expression) *ast.Expression

    // Go: printer/factory.go:229 NewStrictEqualityExpression

    // Go: printer/factory.go:233 NewStrictInequalityExpression

    //
    // Compound Nodes
    //

    // Go: printer/factory.go:241 NewVoidZeroExpression
}

// Go: printer/factory.go:245 flattenCommaElement

// Go: printer/factory.go:255 flattenCommaElements

impl NodeFactory {
    // Go: printer/factory.go:264 InlineExpressions

    //
    // Utilities
    //

    // Go: printer/factory.go:283 CreateExpressionFromEntityName

    // Go: printer/factory.go:301 RestoreEnclosingLabel

    // Go: printer/factory.go:317 CreateForOfBindingStatement

    // Go: printer/factory.go:345 NewTypeCheck

    // Go: printer/factory.go:355 NewMethodCall

    // Go: printer/factory.go:375 NewGlobalMethodCall

    // Go: printer/factory.go:379 NewFunctionCallCall

    // Go: printer/factory.go:387 NewArraySliceCall

    // Go: printer/factory.go:407 isIgnorableParen

    // Go: printer/factory.go:416 updateOuterExpression

    // Go: printer/factory.go:437 RestoreOuterExpressions

    // Go: printer/factory.go:448 EnsureUseStrict

    // Go: printer/factory.go:462 SplitStandardPrologue

    // Go: printer/factory.go:472 SplitCustomPrologue
}

//
// Declaration Names
//

// Go: printer/factory.go:485 NameOptions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NameOptions {
    /// indicates whether comments may be emitted for the name.
    pub allow_comments: bool,
    /// indicates whether source maps may be emitted for the name.
    pub allow_source_maps: bool,
}

// Go: printer/factory.go:490 AssignedNameOptions
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AssignedNameOptions {
    /// indicates whether comments may be emitted for the name.
    pub allow_comments: bool,
    /// indicates whether source maps may be emitted for the name.
    pub allow_source_maps: bool,
    /// indicates whether the assigned name of a declaration shouldn't be considered.
    pub ignore_assigned_name: bool,
}

impl NodeFactory {
    // Go: printer/factory.go:496 getName

    // Go: printer/factory.go:524 GetLocalName

    // Go: printer/factory.go:531 GetLocalNameEx

    // Go: printer/factory.go:539 GetExportName

    // Go: printer/factory.go:547 GetExportNameEx

    // Go: printer/factory.go:552 GetDeclarationName

    // Go: printer/factory.go:557 GetDeclarationNameEx

    // Go: printer/factory.go:561 GetNamespaceMemberName

    // Go: printer/factory.go:580 GetExternalModuleOrNamespaceExportName

    //
    // Emit Helpers
    //

    // Go: printer/factory.go:593 NewUnscopedHelperName

    // TypeScript Helpers

    // Go: printer/factory.go:601 NewDecorateHelper

    // Go: printer/factory.go:623 NewMetadataHelper

    // Go: printer/factory.go:638 NewParamHelper

    // ESNext Helpers

    // Go: printer/factory.go:653 NewAddDisposableResourceHelper

    // Go: printer/factory.go:664 NewDisposeResourcesHelper
}

// Class Fields Helpers

// Go: printer/factory.go:677 PrivateIdentifierKind
/// Go `type PrivateIdentifierKind string`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PrivateIdentifierKind(pub &'static str);

impl PrivateIdentifierKind {
    pub const FIELD: Self = Self("f");
    pub const METHOD: Self = Self("m");
    pub const ACCESSOR: Self = Self("a");
    pub const UNTRANSFORMED: Self = Self("untransformed");
}

impl NodeFactory {
    // Go: printer/factory.go:686 NewClassPrivateFieldGetHelper

    // Go: printer/factory.go:703 NewClassPrivateFieldSetHelper

    // Go: printer/factory.go:720 NewClassPrivateFieldInHelper

    // Go: printer/factory.go:732 NewObjectDefinePropertyCall

    // Go: printer/factory.go:748 NewReflectGetCall

    // Go: printer/factory.go:764 NewReflectSetCall

    // Go: printer/factory.go:780 NewFunctionBindCall

    // Go: printer/factory.go:788 NewImmediatelyInvokedArrowFunction

    // Go: printer/factory.go:808 NewExportDefault

    // Go: printer/factory.go:813 NewExternalModuleExport

    // ES2018 Helpers

    // Go: printer/factory.go:821 NewAssignHelper

    // ES2018 Destructuring Helpers

    // Go: printer/factory.go:827 NewRestHelper
    // PORT: Go `computedTempVariables == nil` is `None`.

    // ES2018 Helpers

    // Go: printer/factory.go:871 NewAwaitHelper

    // Go: printer/factory.go:883 NewAsyncGeneratorHelper

    // Go: printer/factory.go:914 NewAsyncDelegatorHelper

    // Go: printer/factory.go:927 NewAsyncValuesHelper

    // !!! ES2017 Helpers

    // Go: printer/factory.go:941 NewAwaiterHelper

    // ES Decorator Helpers

    // Go: printer/factory.go:1000 NewESDecorateClassContextObject

    // Go: printer/factory.go:1009 NewESDecorateClassElementAccessGetMethod

    // Go: printer/factory.go:1033 NewESDecorateClassElementAccessSetMethod

    // Go: printer/factory.go:1062 NewESDecorateClassElementAccessHasMethod

    // Go: printer/factory.go:1098 NewESDecorateClassElementAccessObject

    // Go: printer/factory.go:1122 NewESDecorateClassElementContextObject

    // Go: printer/factory.go:1168 NewESDecorateHelper

    // Go: printer/factory.go:1179 NewRunInitializersHelper

    // ES2015 Helpers

    // Go: printer/factory.go:1198 NewTemplateObjectHelper

    // Go: printer/factory.go:1209 NewPropKeyHelper

    // Go: printer/factory.go:1220 NewSetFunctionNameHelper

    // ES Module Helpers

    // Go: printer/factory.go:1240 NewImportDefaultHelper

    // Go: printer/factory.go:1252 NewImportStarHelper

    // Go: printer/factory.go:1264 NewExportStarHelper

    // Go: printer/factory.go:1275 NewAssignmentTargetWrapper

    // Go: printer/factory.go:1300 NewRewriteRelativeImportExtensionsHelper
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::AssertUnwindSafe;

    // Go prints `%s` of the Kind in this panic (printer/factory.go:433
    // `panic(fmt.Sprintf("Unexpected outer expression kind: %s",
    // outerExpression.Kind))`), which is `Kind.String()`.
    #[test]
    fn unexpected_outer_expression_panic_names_the_go_kind() {
        let context = super::super::emit_context::new_emit_context();
        let f = NodeFactory::new(&context);
        let (outer, inner) = (f.new_identifier("x"), f.new_identifier("y"));
        let payload =
            std::panic::catch_unwind(AssertUnwindSafe(|| f.update_outer_expression(outer, inner)))
                .expect_err("no panic");
        assert_eq!(
            payload.downcast_ref::<String>().map(String::as_str),
            Some("Unexpected outer expression kind: KindIdentifier")
        );
    }
}
