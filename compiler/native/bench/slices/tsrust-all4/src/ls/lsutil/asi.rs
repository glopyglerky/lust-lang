//! Port of Go `ls/lsutil/asi.go`.

use crate::astnav;
use crate::ls::lsutil::prelude::*;

// Go: ls/lsutil/asi.go:9 PositionIsASICandidate

// Go: ls/lsutil/asi.go:21 SyntaxMayBeASICandidate
pub fn syntax_may_be_asi_candidate(kind: SyntaxKind) -> bool {
    syntax_requires_trailing_comma_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_function_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_module_block_or_semicolon_or_asi(kind)
        || syntax_requires_trailing_semicolon_or_asi(kind)
}

// Go: ls/lsutil/asi.go:28 SyntaxRequiresTrailingCommaOrSemicolonOrASI
pub fn syntax_requires_trailing_comma_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::CallSignature
        || kind == SyntaxKind::ConstructSignature
        || kind == SyntaxKind::IndexSignature
        || kind == SyntaxKind::PropertySignature
        || kind == SyntaxKind::MethodSignature
}

// Go: ls/lsutil/asi.go:36 SyntaxRequiresTrailingFunctionBlockOrSemicolonOrASI
pub fn syntax_requires_trailing_function_block_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::FunctionDeclaration
        || kind == SyntaxKind::Constructor
        || kind == SyntaxKind::MethodDeclaration
        || kind == SyntaxKind::GetAccessor
        || kind == SyntaxKind::SetAccessor
}

// Go: ls/lsutil/asi.go:44 SyntaxRequiresTrailingModuleBlockOrSemicolonOrASI
pub fn syntax_requires_trailing_module_block_or_semicolon_or_asi(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::ModuleDeclaration
}

// Go: ls/lsutil/asi.go:48 SyntaxRequiresTrailingSemicolonOrASI
pub fn syntax_requires_trailing_semicolon_or_asi(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::VariableStatement
        || kind == SyntaxKind::ExpressionStatement
        || kind == SyntaxKind::DoStatement
        || kind == SyntaxKind::ContinueStatement
        || kind == SyntaxKind::BreakStatement
        || kind == SyntaxKind::ReturnStatement
        || kind == SyntaxKind::ThrowStatement
        || kind == SyntaxKind::DebuggerStatement
        || kind == SyntaxKind::PropertyDeclaration
        || kind == SyntaxKind::TypeAliasDeclaration
        || kind == SyntaxKind::ImportDeclaration
        || kind == SyntaxKind::ImportEqualsDeclaration
        || kind == SyntaxKind::ExportDeclaration
        || kind == SyntaxKind::NamespaceExportDeclaration
        || kind == SyntaxKind::ExportAssignment
}

// Go: ls/lsutil/asi.go:66 NodeIsASICandidate
