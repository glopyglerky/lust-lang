//! Go `internal/compiler/program.go` lines 724 to 1289: compiler option
//! checks (`verifyCompilerOptions`), project reference checks and emit
//! blocking.

use crate::frontend::prelude::*;

/// The Go `core.Memoize` locals of `verifyCompilerOptions`.
// PORT: Go computes them lazily. They only read the config file syntax and
// have no side effects, so they are computed once at the start.
struct OptionsSyntax {
    // Go `sourceFile()`. `Node::NIL` for the Go nil.
    source_file: Node,
    // Go `configFilePath()`.
    config_file_path: String,
    // Go `getCompilerOptionsPropertySyntax()`. `None` for the Go nil.
    compiler_options_property: Option<Node>,
    // Go `getCompilerOptionsObjectLiteralSyntax()`. `Node::NIL` for the Go nil.
    compiler_options_object_literal: Node,
}

impl OptionsSyntax {
}


// PORT: the Go closures of `verifyCompilerOptions` append to
// `p.programDiagnostics` and return the `*ast.Diagnostic`. Here they are
// methods that return the index of the new diagnostic in
// `program_diagnostics`, so a caller can still add a message chain.

// Go: program.go:1431 hasZeroOrOneAsteriskCharacter
pub fn has_zero_or_one_asterisk_character(str: &str) -> bool {
    let mut seen_asterisk = false;
    for ch in str.chars() {
        if ch == '*' {
            if !seen_asterisk {
                seen_asterisk = true;
            } else {
                // have already seen asterisk
                return false;
            }
        }
    }
    true
}

// Go: program.go:1446 moduleResolutionSupportsPackageJsonExportsAndImports
pub fn module_resolution_supports_package_json_exports_and_imports(
    module_resolution: ModuleResolutionKind,
) -> bool {
    module_resolution >= ModuleResolutionKind::NODE16
        && module_resolution <= ModuleResolutionKind::NODE_NEXT
        || module_resolution == ModuleResolutionKind::BUNDLER
}

// Go: program.go:1451 emitModuleKindIsNonNodeESM
pub fn emit_module_kind_is_non_node_esm(module_kind: ModuleKind) -> bool {
    module_kind >= ModuleKind::ES2015 && module_kind <= ModuleKind::ES_NEXT
}
