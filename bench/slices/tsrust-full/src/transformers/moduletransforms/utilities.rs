//! Port of Go `transformers/moduletransforms/utilities.go`.

use crate::frontend::outputpaths::get_output_extension;
use crate::frontend::tspath;
use crate::prelude::*;

// Go: transformers/moduletransforms/utilities.go:12 isDeclarationNameOfEnumOrNamespace

// Go: transformers/moduletransforms/utilities.go:22 rewriteModuleSpecifier

// Go: core/core.go:724 ShouldRewriteModuleSpecifier
// PORT: the checker copy (`checker_p17::core_p17`) is private, so this unit
// keeps its own copy of the Go `core` helper.

// Go: transformers/moduletransforms/utilities.go:36 createEmptyImports

// Go: transformers/moduletransforms/utilities.go:53 getExternalModuleNameLiteral

// Go: transformers/moduletransforms/utilities.go:74 tryGetModuleNameFromFile

// Go: transformers/moduletransforms/utilities.go:85 tryGetModuleNameFromDeclaration

// Go: transformers/moduletransforms/utilities.go:93 getExternalModuleNameFromPath
/// Resolves a local path to a path which is absolute to the base of the emit
pub(crate) fn get_external_module_name_from_path(
    _file_name: &str,
    _reference_path: &str,
) -> String {
    // !!!
    String::new()
}

// Go: transformers/moduletransforms/utilities.go:100 tryRenameExternalModule
/// Some bundlers (SystemJS builder) sometimes want to rename dependencies.
/// Here we check if alternative name was provided for a given moduleName and return it if possible.
pub(crate) fn try_rename_external_module(
    _factory: &NodeFactory,
    _module_name: Node,
    _source_file: Node,
) -> Node {
    // !!!
    Node::NIL
}

// Go: transformers/moduletransforms/utilities.go:105 isFileLevelReservedGeneratedIdentifier

// Go: transformers/moduletransforms/utilities.go:116 isSimpleInlineableExpression





// Go: ast/utilities.go IsEffectiveExternalModule
