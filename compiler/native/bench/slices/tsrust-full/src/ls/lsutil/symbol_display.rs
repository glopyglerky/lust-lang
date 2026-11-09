//! Port of Go `ls/lsutil/symbol_display.go`.
//!
//! PORT: Go passes a `*checker.Checker` that may be nil and reads symbol
//! fields through the symbol pointer. Rust reads them from an arena, so the
//! checker parameter is `TypeChecker`: a checker (its arena is
//! `Checker.symbols`), or a nil checker with the arena that holds the
//! caller's symbols.

use crate::ls::lsutil::prelude::*;

use crate::flags_macros::{go_enum, go_flags};

// Go: ls/lsutil/symbol_display.go:10 ScriptElementKind
go_enum!(ScriptElementKind, i32 {
    UNKNOWN = 0; // ScriptElementKindUnknown
    WARNING = 1; // ScriptElementKindWarning
    // predefined type (void) or keyword (class)
    KEYWORD = 2; // ScriptElementKindKeyword
    // top level script node
    SCRIPT_ELEMENT = 3; // ScriptElementKindScriptElement
    // module foo {}
    MODULE_ELEMENT = 4; // ScriptElementKindModuleElement
    // class X {}
    CLASS_ELEMENT = 5; // ScriptElementKindClassElement
    // var x = class X {}
    LOCAL_CLASS_ELEMENT = 6; // ScriptElementKindLocalClassElement
    // interface Y {}
    INTERFACE_ELEMENT = 7; // ScriptElementKindInterfaceElement
    // type T = ...
    TYPE_ELEMENT = 8; // ScriptElementKindTypeElement
    // enum E {}
    ENUM_ELEMENT = 9; // ScriptElementKindEnumElement
    ENUM_MEMBER_ELEMENT = 10; // ScriptElementKindEnumMemberElement
    // Inside module and script only.
    // const v = ...
    VARIABLE_ELEMENT = 11; // ScriptElementKindVariableElement
    // Inside function.
    LOCAL_VARIABLE_ELEMENT = 12; // ScriptElementKindLocalVariableElement
    // using foo = ...
    VARIABLE_USING_ELEMENT = 13; // ScriptElementKindVariableUsingElement
    // await using foo = ...
    VARIABLE_AWAIT_USING_ELEMENT = 14; // ScriptElementKindVariableAwaitUsingElement
    // Inside module and script only.
    // function f() {}
    FUNCTION_ELEMENT = 15; // ScriptElementKindFunctionElement
    // Inside function.
    LOCAL_FUNCTION_ELEMENT = 16; // ScriptElementKindLocalFunctionElement
    // class X { [public|private]* foo() {} }
    MEMBER_FUNCTION_ELEMENT = 17; // ScriptElementKindMemberFunctionElement
    // class X { [public|private]* [get|set] foo:number; }
    MEMBER_GET_ACCESSOR_ELEMENT = 18; // ScriptElementKindMemberGetAccessorElement
    MEMBER_SET_ACCESSOR_ELEMENT = 19; // ScriptElementKindMemberSetAccessorElement
    // class X { [public|private]* foo:number; }
    // interface Y { foo:number; }
    MEMBER_VARIABLE_ELEMENT = 20; // ScriptElementKindMemberVariableElement
    // class X { [public|private]* accessor foo: number; }
    MEMBER_ACCESSOR_VARIABLE_ELEMENT = 21; // ScriptElementKindMemberAccessorVariableElement
    // class X { constructor() { } }
    // class X { static { } }
    CONSTRUCTOR_IMPLEMENTATION_ELEMENT = 22; // ScriptElementKindConstructorImplementationElement
    // interface Y { ():number; }
    CALL_SIGNATURE_ELEMENT = 23; // ScriptElementKindCallSignatureElement
    // interface Y { []:number; }
    INDEX_SIGNATURE_ELEMENT = 24; // ScriptElementKindIndexSignatureElement
    // interface Y { new():Y; }
    CONSTRUCT_SIGNATURE_ELEMENT = 25; // ScriptElementKindConstructSignatureElement
    // function foo(*Y*: string)
    PARAMETER_ELEMENT = 26; // ScriptElementKindParameterElement
    TYPE_PARAMETER_ELEMENT = 27; // ScriptElementKindTypeParameterElement
    PRIMITIVE_TYPE = 28; // ScriptElementKindPrimitiveType
    LABEL = 29; // ScriptElementKindLabel
    ALIAS = 30; // ScriptElementKindAlias
    CONST_ELEMENT = 31; // ScriptElementKindConstElement
    LET_ELEMENT = 32; // ScriptElementKindLetElement
    DIRECTORY = 33; // ScriptElementKindDirectory
    EXTERNAL_MODULE_NAME = 34; // ScriptElementKindExternalModuleName
    // String literal
    STRING = 35; // ScriptElementKindString
    // Jsdoc @link: in `{@link C link text}`, the before and after text "{@link " and "}"
    LINK = 36; // ScriptElementKindLink
    // Jsdoc @link: in `{@link C link text}`, the entity name "C"
    LINK_NAME = 37; // ScriptElementKindLinkName
    // Jsdoc @link: in `{@link C link text}`, the link text "link text"
    LINK_TEXT = 38; // ScriptElementKindLinkText
});

// Go: ls/lsutil/symbol_display.go:85 ScriptElementKindModifier
// PORT: Go starts the `1 << iota` run on the second line, so `Public` is
// `1 << 1`.
go_flags!(ScriptElementKindModifier, u32 {
    NONE = 0; // ScriptElementKindModifierNone
    PUBLIC = 1 << 1; // ScriptElementKindModifierPublic
    PRIVATE = 1 << 2; // ScriptElementKindModifierPrivate
    PROTECTED = 1 << 3; // ScriptElementKindModifierProtected
    EXPORTED = 1 << 4; // ScriptElementKindModifierExported
    AMBIENT = 1 << 5; // ScriptElementKindModifierAmbient
    STATIC = 1 << 6; // ScriptElementKindModifierStatic
    ABSTRACT = 1 << 7; // ScriptElementKindModifierAbstract
    OPTIONAL = 1 << 8; // ScriptElementKindModifierOptional
    DEPRECATED = 1 << 9; // ScriptElementKindModifierDeprecated
    DTS = 1 << 10; // ScriptElementKindModifierDts
    TS = 1 << 11; // ScriptElementKindModifierTs
    TSX = 1 << 12; // ScriptElementKindModifierTsx
    JS = 1 << 13; // ScriptElementKindModifierJs
    JSX = 1 << 14; // ScriptElementKindModifierJsx
    JSON = 1 << 15; // ScriptElementKindModifierJson
    DMTS = 1 << 16; // ScriptElementKindModifierDmts
    MTS = 1 << 17; // ScriptElementKindModifierMts
    MJS = 1 << 18; // ScriptElementKindModifierMjs
    DCTS = 1 << 19; // ScriptElementKindModifierDcts
    CTS = 1 << 20; // ScriptElementKindModifierCts
    CJS = 1 << 21; // ScriptElementKindModifierCjs
});

/// One entry of Go `scriptElementKindModifierNames`.
struct ScriptElementKindModifierName {
    flag: ScriptElementKindModifier,
    name: &'static str,
}

// Go: ls/lsutil/symbol_display.go:112 scriptElementKindModifierNames
const SCRIPT_ELEMENT_KIND_MODIFIER_NAMES: [ScriptElementKindModifierName; 21] = [
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::PUBLIC,
        name: "public",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::PRIVATE,
        name: "private",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::PROTECTED,
        name: "protected",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::EXPORTED,
        name: "export",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::AMBIENT,
        name: "declare",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::STATIC,
        name: "static",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::ABSTRACT,
        name: "abstract",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::OPTIONAL,
        name: "optional",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::DEPRECATED,
        name: "deprecated",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::DTS,
        name: ".d.ts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::TS,
        name: ".ts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::TSX,
        name: ".tsx",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::JS,
        name: ".js",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::JSX,
        name: ".jsx",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::JSON,
        name: ".json",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::DMTS,
        name: ".d.mts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::MTS,
        name: ".mts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::MJS,
        name: ".mjs",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::DCTS,
        name: ".d.cts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::CTS,
        name: ".cts",
    },
    ScriptElementKindModifierName {
        flag: ScriptElementKindModifier::CJS,
        name: ".cjs",
    },
];

impl ScriptElementKindModifier {
    // Go: ls/lsutil/symbol_display.go:139 Strings
    // PORT: Go `collections.Set[string]` is `FxHashSet<String>`.
    #[must_use]
    pub fn strings(self) -> FxHashSet<String> {
        let mut result = FxHashSet::default();
        for entry in &SCRIPT_ELEMENT_KIND_MODIFIER_NAMES {
            if self.intersects(entry.flag) {
                result.insert(entry.name.to_string());
            }
        }
        result
    }
}

// Go: ls/lsutil/symbol_display.go:149 FileExtensionKindModifiers




// Go: ls/lsutil/symbol_display.go:162 GetSymbolKind

// Go: ls/lsutil/symbol_display.go:200 getSymbolKindOfConstructorPropertyMethodAccessorFunctionOrVar

// Go: ls/lsutil/symbol_display.go:299 isFirstDeclarationOfSymbolParameter

// Go: ls/lsutil/symbol_display.go:317 isLocalVariableOrFunction

// Go: ls/lsutil/symbol_display.go:349 GetSymbolModifiers

// Go: ls/lsutil/symbol_display.go:368 getNormalizedSymbolModifiers

// Go: ls/lsutil/symbol_display.go:388 isDeprecatedDeclaration
// PORT: Go unexported and used only in this file. It is `pub` because the
// lsutil prelude picks it over `ast::is_deprecated_declaration`.

// Go: ls/lsutil/symbol_display.go:395 getNodeModifiers
