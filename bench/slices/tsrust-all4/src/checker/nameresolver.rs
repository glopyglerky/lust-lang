//! Port of `binder/nameresolver.go`.
//!
//! PORT: Go `binder.NameResolver` holds plain Go closures bound to the
//! checker. Here each callback is an `Option<Rc<dyn Fn(&mut Checker, ...)>>`
//! (nil func = `None`), and `resolve` takes the checker explicitly. The
//! resolver's own methods take `&self` so a resolver kept in an `Rc` can be
//! re-entered from its callbacks (Go calls `Resolve` recursively through the
//! checker). The only mutable resolver field, `ArgumentsSymbol`, is a `Cell`.
//!
//! PORT: Go reads symbols through their pointers. Here `resolve` reads them
//! through a `NameResolverHost`: the checker, or the program's binder
//! symbols for the binder reference resolver of JS emit, which runs on
//! threads with no checker (`transformers::reference_resolver`).

use crate::prelude::*;
use std::cell::Cell;

// PORT: Go `ast.InternalSymbolNameDefault` ("default"). Kept private here so
// this file does not depend on the constant's Rust name in the ast files.
const INTERNAL_SYMBOL_NAME_DEFAULT: &str = "default";







thread_local! {
    /// U1 (a): the text and name of the last `resolver_name_text` call on
    /// this thread, for the next `NameResolver::resolve`.
    static NAME_HINT: Cell<Option<(&'static str, Name)>> = const { Cell::new(None) };
}



// Go: binder/nameresolver.go:450 GetLocalSymbolForExportDefault

// Go: binder/nameresolver.go:463 isExportDefaultSymbol

// Go: binder/nameresolver.go:467 getIsDeferredContext
// PERF: `kind` is `location.kind()`, which the caller has read (see
// `resolve_name`). `location` is not nil.

// Go: binder/nameresolver.go:485 isTypeParameterSymbolDeclaredInContainer

/// True for the kinds of the locals containers (`locals_container_variants!`
/// in `ast/node.rs`, Go `LocalsContainerData`): only a node of one of these
/// kinds can have locals.
pub fn is_locals_container_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ArrowFunction
            | SyntaxKind::Block
            | SyntaxKind::CallSignature
            | SyntaxKind::CaseBlock
            | SyntaxKind::CatchClause
            | SyntaxKind::ClassDeclaration
            | SyntaxKind::ClassExpression
            | SyntaxKind::ClassStaticBlockDeclaration
            | SyntaxKind::ConditionalType
            | SyntaxKind::ConstructSignature
            | SyntaxKind::Constructor
            | SyntaxKind::ConstructorType
            | SyntaxKind::ForInStatement
            | SyntaxKind::ForOfStatement
            | SyntaxKind::ForStatement
            | SyntaxKind::FunctionDeclaration
            | SyntaxKind::FunctionExpression
            | SyntaxKind::FunctionType
            | SyntaxKind::GetAccessor
            | SyntaxKind::IndexSignature
            | SyntaxKind::JsDocSignature
            | SyntaxKind::MappedType
            | SyntaxKind::MethodDeclaration
            | SyntaxKind::MethodSignature
            | SyntaxKind::ModuleDeclaration
            | SyntaxKind::SetAccessor
            | SyntaxKind::SourceFile
            | SyntaxKind::TypeAliasDeclaration
            | SyntaxKind::JsTypeAliasDeclaration
    )
}

// Go: binder/nameresolver.go:497 isSelfReferenceLocation
// PERF: `kind` is `node.kind()`, which the caller has read.
