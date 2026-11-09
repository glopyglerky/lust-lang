// K1 compatibility fixture: go_flags! and go_enum! output types.
// panic-mode: abort
//
// Real shape: crates/ts_goport/src/flags_macros.rs (the macros, copied
// verbatim below) and NodeFlags plus LanguageVariant from
// crates/ts_goport/src/flags.rs. 66 ts-rust types use these macros.
// lust must expand macro_rules with const blocks, operator impls and
// Debug for the generated types, with the same bits as rustc.
//
// The macros are macro_export in ts-rust; here macro_rules definitions
// at file scope serve the same uses in one file.

//! Newtype macros for Go integer flag and enum types.
//!
//! `go_flags!` makes a bit set. `go_enum!` makes a plain Go enum. Both keep
//! the Go numeric values, so tables and switches port one to one.
//!
//! Both are `#[macro_export]` in `goport_util`. `ts_goport` files import
//! them from `crate::flags_macros` (an inline module in its `lib.rs`).

#[macro_export]
macro_rules! go_flags {
    ($name:ident, $repr:ty { $($(#[$meta:meta])* $konst:ident = $value:expr;)* }) => {
        #[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        impl $name {
            $($(#[$meta])* pub const $konst: Self = Self($value as $repr);)*

            #[must_use]
            pub const fn bits(self) -> $repr {
                self.0
            }

            /// Go `x&y != 0`.
            #[must_use]
            pub const fn intersects(self, other: Self) -> bool {
                self.0 & other.0 != 0
            }

            /// Go `x&y == y`.
            #[must_use]
            pub const fn contains(self, other: Self) -> bool {
                self.0 & other.0 == other.0
            }

            #[must_use]
            pub const fn is_empty(self) -> bool {
                self.0 == 0
            }

            /// Go `x &^ y`.
            #[must_use]
            pub const fn without(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }

            #[must_use]
            pub const fn union(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, rhs: Self) -> Self {
                Self(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, rhs: Self) -> Self {
                Self(self.0 & rhs.0)
            }
        }

        impl std::ops::BitAndAssign for $name {
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl std::ops::Not for $name {
            type Output = Self;
            fn not(self) -> Self {
                Self(!self.0)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({:#x})", stringify!($name), self.0)
            }
        }
    };
}

#[macro_export]
macro_rules! go_enum {
    ($name:ident, $repr:ty { $($(#[$meta:meta])* $konst:ident = $value:expr;)* }) => {
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub $repr);

        #[allow(non_upper_case_globals)]
        impl $name {
            $($(#[$meta])* pub const $konst: Self = Self($value as $repr);)*
        }
    };
}

go_flags!(NodeFlags, u32 {
    NONE = 0; // NodeFlagsNone
    LET = 1; // NodeFlagsLet
    CONST = 2; // NodeFlagsConst
    USING = 4; // NodeFlagsUsing
    REPARSED = 8; // NodeFlagsReparsed
    SYNTHESIZED = 0x10; // NodeFlagsSynthesized
    OPTIONAL_CHAIN = 0x20; // NodeFlagsOptionalChain
    EXPORT_CONTEXT = 0x40; // NodeFlagsExportContext
    CONTAINS_THIS = 0x80; // NodeFlagsContainsThis
    HAS_IMPLICIT_RETURN = 0x100; // NodeFlagsHasImplicitReturn
    HAS_EXPLICIT_RETURN = 0x200; // NodeFlagsHasExplicitReturn
    DISALLOW_IN_CONTEXT = 0x400; // NodeFlagsDisallowInContext
    YIELD_CONTEXT = 0x800; // NodeFlagsYieldContext
    DECORATOR_CONTEXT = 0x1000; // NodeFlagsDecoratorContext
    AWAIT_CONTEXT = 0x2000; // NodeFlagsAwaitContext
    DISALLOW_CONDITIONAL_TYPES_CONTEXT = 0x4000; // NodeFlagsDisallowConditionalTypesContext
    THIS_NODE_HAS_ERROR = 0x8000; // NodeFlagsThisNodeHasError
    JAVA_SCRIPT_FILE = 0x10000; // NodeFlagsJavaScriptFile
    THIS_NODE_OR_ANY_SUB_NODES_HAS_ERROR = 0x20000; // NodeFlagsThisNodeOrAnySubNodesHasError
    HAS_ASYNC_FUNCTIONS = 0x40000; // NodeFlagsHasAsyncFunctions
    POSSIBLY_CONTAINS_DYNAMIC_IMPORT = 0x80000; // NodeFlagsPossiblyContainsDynamicImport
    POSSIBLY_CONTAINS_IMPORT_META = 0x100000; // NodeFlagsPossiblyContainsImportMeta
    HAS_JS_DOC = 0x200000; // NodeFlagsHasJSDoc
    JS_DOC = 0x400000; // NodeFlagsJSDoc
    AMBIENT = 0x800000; // NodeFlagsAmbient
    IN_WITH_STATEMENT = 0x1000000; // NodeFlagsInWithStatement
    JSON_FILE = 0x2000000; // NodeFlagsJsonFile
    POSSIBLY_CONTAINS_DEPRECATED_TAG = 0x4000000; // NodeFlagsPossiblyContainsDeprecatedTag
    UNREACHABLE = 0x8000000; // NodeFlagsUnreachable
    REPARSER_TRANSFORMED_LITERAL = 0x10000000; // NodeFlagsReparserTransformedLiteral
    BLOCK_SCOPED = 7; // NodeFlagsBlockScoped
    CONSTANT = 6; // NodeFlagsConstant
    AWAIT_USING = 6; // NodeFlagsAwaitUsing
    REACHABILITY_CHECK_FLAGS = 0x300; // NodeFlagsReachabilityCheckFlags
    REACHABILITY_AND_EMIT_FLAGS = 0x40300; // NodeFlagsReachabilityAndEmitFlags
    CONTEXT_FLAGS = 0x1817c00; // NodeFlagsContextFlags
    TYPE_EXCLUDES_FLAGS = 0x2800; // NodeFlagsTypeExcludesFlags
    PERMANENTLY_SET_INCREMENTAL_FLAGS = 0x180000; // NodeFlagsPermanentlySetIncrementalFlags
    IDENTIFIER_HAS_EXTENDED_UNICODE_ESCAPE = 0x80; // NodeFlagsIdentifierHasExtendedUnicodeEscape
    IDENTIFIER_IS_IN_JS_DOC_NAMESPACE = 0x40000; // NodeFlagsIdentifierIsInJSDocNamespace
    NESTED_NAMESPACE = 0x20; // NodeFlagsNestedNamespace
});

go_enum!(LanguageVariant, i32 {
    STANDARD = 0; // LanguageVariantStandard
    JSX = 1; // LanguageVariantJSX
});

fn main() {
    let none = NodeFlags::NONE;
    let mut f = NodeFlags::LET | NodeFlags::AWAIT_CONTEXT;
    println!("none bits={} empty={}", none.bits(), none.is_empty());
    println!("f bits={:#x}", f.bits());
    println!("has let={} has const={}", f.contains(NodeFlags::LET), f.contains(NodeFlags::CONST));
    println!("meets await={}", f.intersects(NodeFlags::AWAIT_CONTEXT | NodeFlags::YIELD_CONTEXT));
    f |= NodeFlags::SYNTHESIZED;
    f &= !NodeFlags::LET;
    println!("after ops bits={:#x}", f.bits());
    println!("default={:?}", NodeFlags::default().bits());
    println!("debug: {:?}", NodeFlags::CONST);
    let ordered = [NodeFlags::CONST, NodeFlags::LET, NodeFlags::NONE];
    let mut sorted = ordered;
    sorted.sort();
    println!("sorted: {:?}", sorted.iter().map(|x| x.bits()).collect::<Vec<_>>());
    // The checker masks context flags exactly like this.
    let ctx = NodeFlags::AWAIT_CONTEXT | NodeFlags::YIELD_CONTEXT | NodeFlags::DECORATOR_CONTEXT;
    println!("ctx &= f -> {:#x}", (ctx & f).bits());

    let v = LanguageVariant::JSX;
    println!("variant jsx={} standard={} eq={}", v.0, LanguageVariant::STANDARD.0, v == LanguageVariant::JSX);
    println!("variant debug: {:?}", v);
}
