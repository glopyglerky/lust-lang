//! Go package `printer`: the small type files.
//!
//! Ports `emitflags.go`, `generatedidentifierflags.go`, `emittextwriter.go`,
//! `emitresolver.go`, `emithost.go` and `sourcefilemetadataprovider.go`.

use crate::flags_macros::{go_enum, go_flags};
use crate::prelude::*;

// ──────────────────────────────────────────────────────────────────────
// emitflags.go
// ──────────────────────────────────────────────────────────────────────

// Go: printer/emitflags.go:3 EmitFlags
go_flags!(EmitFlags, u32 {
    NONE = 0; // EFNone
    SINGLE_LINE = 1 << 0; // The contents of this node should be emitted on a single line.
    MULTI_LINE = 1 << 1; // The contents of this node should be emitted on multiple lines.
    NO_LEADING_SOURCE_MAP = 1 << 2; // Do not emit a leading source map location for this node.
    NO_TRAILING_SOURCE_MAP = 1 << 3; // Do not emit a trailing source map location for this node.
    NO_NESTED_SOURCE_MAPS = 1 << 4; // Do not emit source map locations for children of this node.
    NO_TOKEN_LEADING_SOURCE_MAPS = 1 << 5; // Do not emit leading source map location for token nodes.
    NO_TOKEN_TRAILING_SOURCE_MAPS = 1 << 6; // Do not emit trailing source map location for token nodes.
    NO_LEADING_COMMENTS = 1 << 7; // Do not emit leading comments for this node.
    NO_TRAILING_COMMENTS = 1 << 8; // Do not emit trailing comments for this node.
    NO_NESTED_COMMENTS = 1 << 9; // Do not emit nested comments for children of this node.
    HELPER_NAME = 1 << 10; // The Identifier refers to an *unscoped* emit helper (one that is emitted at the top of the file)
    EXPORT_NAME = 1 << 11; // Ensure an export prefix is added for an identifier that points to an exported declaration with a local name (see SymbolFlags.ExportHasLocal).
    LOCAL_NAME = 1 << 12; // Ensure an export prefix is not added for an identifier that points to an exported declaration.
    INDENTED = 1 << 13; // Adds an explicit extra indentation level for class and function bodies when printing (used to match old emitter).
    NO_INDENTATION = 1 << 14; // Do not indent the node.
    REUSE_TEMP_VARIABLE_SCOPE = 1 << 15; // Reuse the existing temp variable scope during emit.
    CUSTOM_PROLOGUE = 1 << 16; // Treat the statement as if it were a prologue directive (NOTE: Prologue directives are *not* transformed).
    NO_ASCII_ESCAPING = 1 << 17; // When synthesizing nodes that lack an original node or textSourceNode, we want to write the text on the node with ASCII escaping substitutions.
    EXTERNAL_HELPERS = 1 << 18; // This source file has external helpers
    START_ON_NEW_LINE = 1 << 19; // Start this node on a new line
    INDIRECT_CALL = 1 << 20; // Emit CallExpression as an indirect call: `(0, f)()`
    ASYNC_FUNCTION_BODY = 1 << 21; // The node was originally an async function body.
    NO_LEXICAL_ARGUMENTS = 1 << 22; // Do not capture `arguments` for this arrow function.
    TRANSFORM_PRIVATE_STATIC_ELEMENTS = 1 << 23; // Indicates static private elements in a file or class should be transformed regardless of --target (used by esDecorators transform).
    NO_LEXICAL_THIS = 1 << 24; // Do not capture `this` for this node's subtree.
    NO_SOURCE_MAP = (1 << 2) | (1 << 3); // EFNoLeadingSourceMap | EFNoTrailingSourceMap
    NO_TOKEN_SOURCE_MAPS = (1 << 5) | (1 << 6); // EFNoTokenLeadingSourceMaps | EFNoTokenTrailingSourceMaps
    NO_COMMENTS = (1 << 7) | (1 << 8); // EFNoLeadingComments | EFNoTrailingComments
});

// ──────────────────────────────────────────────────────────────────────
// generatedidentifierflags.go
// ──────────────────────────────────────────────────────────────────────

// Go: printer/generatedidentifierflags.go:3 GeneratedIdentifierFlags
go_flags!(GeneratedIdentifierFlags, i32 {
    // Kind
    NONE = 0; // Not automatically generated.
    AUTO = 1; // Automatically generated identifier.
    LOOP = 2; // Automatically generated identifier with a preference for '_i'.
    UNIQUE = 3; // Unique name based on the 'text' property.
    NODE = 4; // Unique name based on the node in the 'Node' property.
    KIND_MASK = 7; // Mask to extract the kind of identifier from its flags.
    // Flags
    RESERVED_IN_NESTED_SCOPES = 1 << 3; // Reserve the generated name in nested scopes
    OPTIMISTIC = 1 << 4; // First instance won't use '_#' if there's no conflict
    FILE_LEVEL = 1 << 5; // Use only the file identifiers list and not generated names to search for conflicts
    ALLOW_NAME_SUBSTITUTION = 1 << 6; // Used by `module.ts` to indicate generated nodes which can have substitutions performed upon them
});

impl GeneratedIdentifierFlags {
    // Go: printer/generatedidentifierflags.go:23 Kind
    #[must_use]
    pub fn kind(self) -> GeneratedIdentifierFlags {
        self & GeneratedIdentifierFlags::KIND_MASK
    }

    // Go: printer/generatedidentifierflags.go:27 IsAuto
    #[must_use]
    pub fn is_auto(self) -> bool {
        self.kind() == GeneratedIdentifierFlags::AUTO
    }

    // Go: printer/generatedidentifierflags.go:31 IsLoop
    #[must_use]
    pub fn is_loop(self) -> bool {
        self.kind() == GeneratedIdentifierFlags::LOOP
    }

    // Go: printer/generatedidentifierflags.go:35 IsUnique
    #[must_use]
    pub fn is_unique(self) -> bool {
        self.kind() == GeneratedIdentifierFlags::UNIQUE
    }

    // Go: printer/generatedidentifierflags.go:39 IsNode
    #[must_use]
    pub fn is_node(self) -> bool {
        self.kind() == GeneratedIdentifierFlags::NODE
    }

    // Go: printer/generatedidentifierflags.go:43 IsReservedInNestedScopes
    #[must_use]
    pub fn is_reserved_in_nested_scopes(self) -> bool {
        self.intersects(GeneratedIdentifierFlags::RESERVED_IN_NESTED_SCOPES)
    }

    // Go: printer/generatedidentifierflags.go:47 IsOptimistic
    #[must_use]
    pub fn is_optimistic(self) -> bool {
        self.intersects(GeneratedIdentifierFlags::OPTIMISTIC)
    }

    // Go: printer/generatedidentifierflags.go:51 IsFileLevel
    #[must_use]
    pub fn is_file_level(self) -> bool {
        self.intersects(GeneratedIdentifierFlags::FILE_LEVEL)
    }

    // Go: printer/generatedidentifierflags.go:55 HasAllowNameSubstitution
    #[must_use]
    pub fn has_allow_name_substitution(self) -> bool {
        self.intersects(GeneratedIdentifierFlags::ALLOW_NAME_SUBSTITUTION)
    }
}

// ──────────────────────────────────────────────────────────────────────
// emittextwriter.go
// ──────────────────────────────────────────────────────────────────────


// ──────────────────────────────────────────────────────────────────────
// emitresolver.go
// ──────────────────────────────────────────────────────────────────────

// Go: printer/emitresolver.go:10 SymbolAccessibility
go_enum!(SymbolAccessibility, i32 {
    ACCESSIBLE = 0; // SymbolAccessibilityAccessible
    NOT_ACCESSIBLE = 1; // SymbolAccessibilityNotAccessible
    CANNOT_BE_NAMED = 2; // SymbolAccessibilityCannotBeNamed
    NOT_RESOLVED = 3; // SymbolAccessibilityNotResolved
});

/// Go `printer.SymbolAccessibilityResult`.
// Go: printer/emitresolver.go:19 SymbolAccessibilityResult
#[derive(Clone, Debug, Default)]
pub struct SymbolAccessibilityResult {
    pub accessibility: SymbolAccessibility,
    pub aliases_to_make_visible: Vec<Node>, // aliases that need to have this symbol visible
    pub error_symbol_name: String,          // Optional - symbol name that results in error
    pub error_node: Node,                   // Optional - node that results in error
    pub error_module_name: String, // Optional - If the symbol is not visible from module, module's name
}

// Go: printer/emitresolver.go:32 TypeReferenceSerializationKind
// Indicates how to serialize the name for a TypeReferenceNode when emitting decorator metadata
go_enum!(TypeReferenceSerializationKind, i32 {
    // The TypeReferenceNode could not be resolved.
    // The type name should be emitted using a safe fallback.
    UNKNOWN = 0;
    // The TypeReferenceNode resolves to a type with a constructor
    // function that can be reached at runtime (e.g. a `class`
    // declaration or a `var` declaration for the static side
    // of a type, such as the global `Promise` type in lib.d.ts).
    TYPE_WITH_CONSTRUCT_SIGNATURE_AND_VALUE = 1;
    // The TypeReferenceNode resolves to a Void-like, Nullable, or Never type.
    VOID_NULLABLE_OR_NEVER_TYPE = 2;
    // The TypeReferenceNode resolves to a Number-like type.
    NUMBER_LIKE_TYPE = 3;
    // The TypeReferenceNode resolves to a BigInt-like type.
    BIG_INT_LIKE_TYPE = 4;
    // The TypeReferenceNode resolves to a String-like type.
    STRING_LIKE_TYPE = 5;
    // The TypeReferenceNode resolves to a Boolean-like type.
    BOOLEAN_TYPE = 6;
    // The TypeReferenceNode resolves to an Array-like type.
    ARRAY_LIKE_TYPE = 7;
    // The TypeReferenceNode resolves to the ESSymbol type.
    ES_SYMBOL_TYPE = 8;
    // The TypeReferenceNode resolved to the global Promise constructor symbol.
    PROMISE = 9;
    // The TypeReferenceNode resolves to a Function type or a type with call signatures.
    TYPE_WITH_CALL_SIGNATURE = 10;
    // The TypeReferenceNode resolves to any other type.
    OBJECT_TYPE = 11;
});



// ──────────────────────────────────────────────────────────────────────
// emithost.go
// ──────────────────────────────────────────────────────────────────────


// ──────────────────────────────────────────────────────────────────────
// sourcefilemetadataprovider.go
// ──────────────────────────────────────────────────────────────────────

/// Go `printer.SourceFileMetaDataProvider`.
// PORT: `tspath.Path` is `&str`; a nil `*ast.SourceFileMetaData` is `None`.
// Go: printer/sourcefilemetadataprovider.go:8 SourceFileMetaDataProvider
pub trait SourceFileMetaDataProvider {
    fn get_source_file_meta_data(&self, path: &str) -> Option<SourceFileMetaData>;
}
