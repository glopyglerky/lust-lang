//! Port of Go `checker/types.go`: `Type`, `TypeData` and every type struct,
//! `Signature`, `IndexInfo`, `TypePredicate`, `TypeAlias`, `ConditionalRoot`,
//! `TupleElementInfo`, and every `*Links` struct.
//!
//! PORT: the Go flag and enum types declared in `types.go` (`ParseFlags`,
//! `SignatureKind`, `ContextFlags`, `TypeFormatFlags`, `SymbolFormatFlags`,
//! `ExternalEmitHelpers`, `VarianceFlags`, `AccessFlags`, `NodeCheckFlags`,
//! `TypeFlags`, `ObjectFlags`, `ElementFlags`, `IndexFlags`,
//! `SignatureFlags`, `TypePredicateKind`, `Ternary`,
//! `MembersOrExportsResolutionKind`) are generated in `crate::flags`. This
//! file only adds the Go constants the generator did not emit.
//!
//! PORT: Go `TypeId` and `SignatureId` are the arena handles in `crate::core`.
//!
//! PORT: all Go struct fields are `pub` so other port files can read and
//! write them like Go code in the same package does.

use crate::jsnum::{Number, PseudoBigInt};
use crate::prelude::*;


/// Number of elements a `SharedList` keeps inline.
const SHARED_LIST_INLINE: usize = 3;

enum SharedListRepr<T: 'static> {
    /// A longer list, in this thread's checker arena. Never freed.
    Arena(&'static [T]),
    /// A longer list of a multi-program process, freed with its last copy.
    /// `SharedList::drop` frees it.
    Owned(Rc<[T]>),
    /// `items[..len]` is the list. Empty is `len == 0`.
    Inline {
        len: InlineLen,
        items: [T; SHARED_LIST_INLINE],
    },
}

/// The length of an inline `SharedList`.
///
/// PERF: an enum and not a `u8`, so the compiler knows that the length is at
/// most `SHARED_LIST_INLINE` and `deref` has no bounds check.
#[derive(Clone, Copy)]
#[repr(u8)]
enum InlineLen {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

// A list must stay 24 bytes, or `Type` grows past 2 cache lines. 32-bit
// targets (wasm32) have smaller pointers and do not check this.



/// Room for the elements of an inline `SharedList` (`SharedList::detach`).
pub type SharedListBuf<T> = [T; SHARED_LIST_INLINE];


/// The clone of an `Owned` list (multi-program only), out of line.
#[cold]
#[inline(never)]
fn clone_owned<T: 'static>(items: &Rc<[T]>) -> SharedListRepr<T> {
    SharedListRepr::Owned(items.clone())
}


/// Drops the `Rc` of an `Owned` list (multi-program only), out of line. The
/// empty arena list put in its place owns nothing.
#[cold]
#[inline(never)]
fn drop_owned<T: 'static>(repr: &mut SharedListRepr<T>) {
    drop(std::mem::replace(repr, SharedListRepr::Arena(&[])));
}





/// Scratch list for the type set of a union or intersection under
/// construction. Most sets have at most 16 members, so they stay on the
/// stack. The created type still stores its own `SharedList` copy.
pub type TypeSet = crate::smallvec::SmallVec<[TypeId; 16]>;











/// Whether new checker data goes in this thread's `CHECKER_ARENA`: only in
/// a one-program process. A multi-program process (watch, language server)
/// drops the checkers of a released program, so there the data is an `Rc`
/// or `Box` that is freed with its checker. Else each released program
/// would leave its arena data behind.
#[inline(always)]
fn use_checker_arena() -> bool {
    !crate::core::is_multi_program()
}


// Go: checker/types.go:36 IndexKind
// PORT: `flags.rs` was generated at an older pin, so this enum (ts#64264) is
// declared here.
crate::flags_macros::go_enum!(IndexKind, i32 {
    STRING = 0; // IndexKindString
    NUMBER = 1; // IndexKindNumber
});

// PORT: Go `TypeFormatFlagsNodeBuilderFlagsMask` is the last constant of the
// `TypeFormatFlags` block (ts#63911). `flags.rs` has only the single-bit
// values, so the mask stays here.
impl TypeFormatFlags {
    pub const NODE_BUILDER_FLAGS_MASK: Self = Self(
        Self::NO_TRUNCATION.0
            | Self::WRITE_ARRAY_AS_GENERIC_TYPE.0
            | Self::GENERATE_NAMES_FOR_SHADOWED_TYPE_PARAMS.0
            | Self::USE_STRUCTURAL_FALLBACK.0
            | Self::WRITE_TYPE_ARGUMENTS_OF_SIGNATURE.0
            | Self::USE_FULLY_QUALIFIED_TYPE.0
            | Self::SUPPRESS_ANY_RETURN_TYPE.0
            | Self::MULTILINE_OBJECT_LITERALS.0
            | Self::WRITE_CLASS_EXPRESSION_AS_TYPE_LITERAL.0
            | Self::USE_TYPE_OF_FUNCTION.0
            | Self::OMIT_PARAMETER_MODIFIERS.0
            | Self::USE_ALIAS_DEFINED_OUTSIDE_CURRENT_SCOPE.0
            | Self::ALLOW_UNIQUE_ES_SYMBOL_TYPE.0
            | Self::IN_TYPE_ALIAS.0
            | Self::USE_INSTANTIATION_EXPRESSIONS.0
            | Self::USE_SINGLE_QUOTES_FOR_STRING_LITERAL_TYPE.0
            | Self::NO_TYPE_REDUCTION.0
            | Self::OMIT_THIS_PARAMETER.0,
    );
}

// Go: checker/types.go:165 externalHelpersModuleNameText
pub const EXTERNAL_HELPERS_MODULE_NAME_TEXT: &str = "tslib";

// Links for referenced symbols

// Go: checker/types.go:176 SymbolReferenceLinks
#[derive(Clone, Debug, Default)]
pub struct SymbolReferenceLinks {
    pub reference_kinds: SymbolFlags, // Flags for the meanings of the symbol that were referenced
}

// Links for value symbols

// Go: checker/types.go:182 ValueSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct ValueSymbolLinks {
    pub resolved_type: TypeId, // Type of value symbol
    pub write_type: TypeId,
    pub target: SymbolId,
    pub mapper: MapperId,
    pub name_type: TypeId,
    pub containing_type: TypeId, // Mapped type for mapped type property, containing union or intersection type for synthetic property
    pub function_or_constructor_checked: bool,
    /// PORT: memo, no Go counterpart. The declaration test of Go
    /// `getTypeOfParameter` (initializer or question token on
    /// `value_declaration`), set on the first call for this symbol. It fits
    /// in the padding after the bool.
    pub optional_parameter: Tristate,
    /// PORT: no Go counterpart. The symbol has its id: a read of
    /// `ValueSymbolLinkStore` gave it (Go keys the store by
    /// `ast.GetSymbolId`), so later reads skip `get_symbol_id`. It fits in
    /// the padding too.
    pub has_id: bool,
}

// The memo bytes must not grow the record.
const _: () = assert!(std::mem::size_of::<ValueSymbolLinks>() == 28);

// Additional links for mapped symbols

// Go: checker/types.go:194 MappedSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct MappedSymbolLinks {
    pub key_type: TypeId,           // Key type for mapped type member
    pub synthetic_origin: SymbolId, // For a property on a mapped or spread type, points back to the original property
}

// Additional links for deferred type symbols

// Go: checker/types.go:201 DeferredSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct DeferredSymbolLinks {
    pub parent: TypeId,            // Source union/intersection of a deferred type
    pub constituents: Vec<TypeId>, // Calculated list of constituents for a deferred type
    pub write_constituents: Vec<TypeId>, // Constituents of a deferred `writeType`
}

// Links for alias symbols

// Go: checker/types.go:209 AliasSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct AliasSymbolLinks {
    pub immediate_target: SymbolId, // Immediate target of an alias. May be another alias. Do not access directly, use `checker.getImmediateAliasedSymbol` instead.
    pub alias_target: SymbolId,     // Resolved (non-alias) target of an alias
    pub referenced: bool, // True if alias symbol has been referenced as a value that can be emitted
    pub type_only_declaration: Node, // First resolved alias declaration that makes the symbol only usable in type constructs
}

// Links for module symbols

// Go: checker/types.go:218 ModuleSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct ModuleSymbolLinks {
    pub resolved_exports: SymbolTable, // Resolved exports of module or combined early- and late-bound static members of a class.
    // PORT: Go nil map is an empty map here. Go only reads it with lookups.
    pub type_only_export_star_map: FxHashMap<String, Node>, // Set on a module symbol when some of its exports were resolved through a 'export type * from "mod"' declaration
    pub exports_checked: bool,
}

// Go: checker/types.go:224 ReverseMappedSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct ReverseMappedSymbolLinks {
    pub property_type: TypeId,
    pub mapped_type: TypeId,     // References a mapped type
    pub constraint_type: TypeId, // References an index type
}

// Links for late-bound symbols

// Go: checker/types.go:232 LateBoundLinks
#[derive(Clone, Debug, Default)]
pub struct LateBoundLinks {
    pub late_symbol: SymbolId,
}

// Links for export type symbols

// Go: checker/types.go:238 ExportTypeLinks
#[derive(Clone, Debug, Default)]
pub struct ExportTypeLinks {
    pub target: SymbolId,         // Target symbol
    pub originating_import: Node, // Import declaration which produced the symbol, present if the symbol is marked as uncallable but had call signatures in `resolveESModuleSymbol`
}

// Links for type aliases

// Go: checker/types.go:245 TypeAliasLinks

// Links for declared types (type parameters, class types, interface types, enums)

// Go: checker/types.go:254 DeclaredTypeLinks
#[derive(Clone, Debug, Default)]
pub struct DeclaredTypeLinks {
    pub declared_type: TypeId,
    pub interface_checked: bool,
    pub index_signatures_checked: bool,
    pub type_parameters_checked: bool,
    pub enum_checked: bool,
}

// Links for switch clauses

// Go: checker/types.go:264 ExhaustiveState
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExhaustiveState(pub u8);

#[allow(non_upper_case_globals)]
impl ExhaustiveState {
    pub const UNKNOWN: Self = Self(0); // Exhaustive state not computed
    pub const COMPUTING: Self = Self(1); // Exhaustive state computation in progress
    pub const FALSE: Self = Self(2); // Switch statement is not exhaustive
    pub const TRUE: Self = Self(3); // Switch statement is exhaustive
}

// Go: checker/types.go:273 SwitchStatementLinks
#[derive(Clone, Debug, Default)]
pub struct SwitchStatementLinks {
    pub exhaustive_state: ExhaustiveState, // Switch statement exhaustiveness
    pub switch_types_computed: bool,
    pub witnesses_computed: bool,
    pub switch_types: Vec<TypeId>,
    pub witnesses: Vec<String>,
}

// Go: checker/types.go:281 ArrayLiteralLinks
#[derive(Clone, Debug, Default)]
pub struct ArrayLiteralLinks {
    pub indices_computed: bool,
    pub first_spread_index: i32, // Index of first spread expression (or -1 if none)
    pub last_spread_index: i32,  // Index of last spread expression (or -1 if none)
}

// Links for late-binding containers

// Go: checker/types.go:296 MembersAndExportsLinks
/// Indexed by `MembersOrExportsResolutionKind` (`links[kind.0 as usize]`).
pub type MembersAndExportsLinks = [SymbolTable; 2];

// Links for synthetic spread properties

// Go: checker/types.go:300 SpreadLinks
#[derive(Clone, Debug, Default)]
pub struct SpreadLinks {
    pub left_spread: SymbolId,  // Left source for synthetic spread property
    pub right_spread: SymbolId, // Right source for synthetic spread property
}

// Links for variances of type aliases and interface types

// Go: checker/types.go:307 VarianceLinks

// Go: checker/types.go:325 MarkedAssignmentSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct MarkedAssignmentSymbolLinks {
    pub last_assignment_pos: i32,
    pub has_definite_assignment: bool, // Symbol is definitely assigned somewhere
}

// Go: checker/types.go:330 accessibleChainCacheKey
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct AccessibleChainCacheKey {
    pub use_only_external_aliasing: bool,
    pub location: Node,
    pub meaning: SymbolFlags,
}

// Go: checker/types.go:336 ContainingSymbolLinks
#[derive(Clone, Debug, Default)]
pub struct ContainingSymbolLinks {
    // PORT: Go keys by `ast.NodeId` of the file node; the `Node` handle of
    // the file is the same identity here.
    pub extended_containers_by_file: FxHashMap<Node, Vec<SymbolId>>, // Symbols of nodes which which logically contain this one, cached by file the request is made within
    // PORT: Go `*[]*ast.Symbol`; nil pointer is `None`.
    pub extended_containers: Option<Vec<SymbolId>>, // Containers (other than the parent) which this symbol is aliased in
    pub accessible_chain_cache: FxHashMap<AccessibleChainCacheKey, Vec<SymbolId>>,
}

// Common links

// Go: checker/types.go:375 NodeLinks
#[derive(Clone, Default)]
pub struct NodeLinks {
    pub flags: NodeCheckFlags, // Set of flags specific to Node
    pub declaration_requires_scope_change: Tristate, // Set by `useOuterVariableScopeInParameter` in checker when downlevel emit would change the name resolution scope inside of a parameter.
    pub has_reported_statement_in_ambient_context: bool, // Cache boolean if we report statements in ambient context
}

// Go: checker/types.go:381 SymbolNodeLinks
#[derive(Clone, Debug, Default)]
pub struct SymbolNodeLinks {
    pub resolved_symbol: SymbolId, // Resolved symbol associated with node
}

// Go: checker/types.go:385 TypeNodeLinks

// Go: checker/types.go:390 ComputedNameNodeLinks
#[derive(Clone, Debug, Default)]
pub struct ComputedNameNodeLinks {
    pub has_name: Option<bool>, // If the node has a computable name. Go `*bool`; nil is `None`.
    pub name: String,           // Resolved name associated with the type of the node
}

/// One ancestor in the `getConditionalFlowTypeOfType` parent walk where a
/// branch of the loop body can add a constraint. Only the tests on the
/// type are left for each call.
#[derive(Clone, Copy, Debug)]
pub enum FlowStep {
    /// The walk comes up from the true type of this `ConditionalType` node.
    /// `covariant` is the walk variance at this step.
    Conditional { parent: Node, covariant: bool },
    /// The walk comes up from the type of this `MappedType` node, which has
    /// no name type.
    Mapped { parent: Node },
}

// Links for enum members

// Go: checker/types.go:397 EnumMemberLinks
#[derive(Clone, Default)]
pub struct EnumMemberLinks {
    pub value: EvaluatorResult, // Constant value of enum member
}

// Links for assertion expressions

// Go: checker/types.go:403 AssertionLinks
#[derive(Clone, Debug, Default)]
pub struct AssertionLinks {
    pub expr_type: TypeId, // Assertion expression type
}

// SourceFile links

// Go: checker/types.go:409 SourceFileLinks
#[derive(Clone, Debug, Default)]
pub struct SourceFileLinks {
    pub type_checked: bool,
    pub unused_checked: bool,
    /// Effect-TS/tsgo: the Effect rules ran on this file (see `check_source_file`).
    pub effect_checked: bool,
    pub external_helpers_module: SymbolId,
    pub requested_external_emit_helpers: ExternalEmitHelpers,
    pub deferred_nodes: IndexSet<Node>,
    pub identifier_check_nodes: Vec<Node>,
    pub local_jsx_namespace: String,
    pub local_jsx_fragment_namespace: String,
    pub local_jsx_factory: Node,
    pub local_jsx_fragment_factory: Node,
    pub jsx_fragment_type: TypeId,
}

// Signature specific links

// Go: checker/types.go:425 SignatureLinks
#[derive(Clone, Debug, Default)]
pub struct SignatureLinks {
    pub resolved_signature: SignatureId, // Cached signature of signature node or call expression
    pub effects_signature: SignatureId,  // Signature with possible control flow effects
    pub decorator_signature: SignatureId, // Signature for decorator as if invoked by the runtime
}

// Go: checker/types.go:520 typeFlagNames
static TYPE_FLAG_NAMES: [(TypeFlags, &str); 29] = [
    (TypeFlags::ANY, "Any"),
    (TypeFlags::UNKNOWN, "Unknown"),
    (TypeFlags::UNDEFINED, "Undefined"),
    (TypeFlags::NULL, "Null"),
    (TypeFlags::VOID, "Void"),
    (TypeFlags::STRING, "String"),
    (TypeFlags::NUMBER, "Number"),
    (TypeFlags::BIG_INT, "BigInt"),
    (TypeFlags::BOOLEAN, "Boolean"),
    (TypeFlags::ES_SYMBOL, "ESSymbol"),
    (TypeFlags::STRING_LITERAL, "StringLiteral"),
    (TypeFlags::NUMBER_LITERAL, "NumberLiteral"),
    (TypeFlags::BIG_INT_LITERAL, "BigIntLiteral"),
    (TypeFlags::BOOLEAN_LITERAL, "BooleanLiteral"),
    (TypeFlags::UNIQUE_ES_SYMBOL, "UniqueESSymbol"),
    (TypeFlags::ENUM_LITERAL, "EnumLiteral"),
    (TypeFlags::ENUM, "Enum"),
    (TypeFlags::NON_PRIMITIVE, "NonPrimitive"),
    (TypeFlags::NEVER, "Never"),
    (TypeFlags::TYPE_PARAMETER, "TypeParameter"),
    (TypeFlags::OBJECT, "Object"),
    (TypeFlags::INDEX, "Index"),
    (TypeFlags::TEMPLATE_LITERAL, "TemplateLiteral"),
    (TypeFlags::STRING_MAPPING, "StringMapping"),
    (TypeFlags::SUBSTITUTION, "Substitution"),
    (TypeFlags::INDEXED_ACCESS, "IndexedAccess"),
    (TypeFlags::CONDITIONAL, "Conditional"),
    (TypeFlags::UNION, "Union"),
    (TypeFlags::INTERSECTION, "Intersection"),
];

// FormatTypeFlags returns the individual flag names as a slice of strings.
// Go: checker/types.go:556 FormatTypeFlags
pub fn format_type_flags(flags: TypeFlags) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(flags.0.count_ones() as usize);
    for (flag, name) in TYPE_FLAG_NAMES.iter() {
        if flags.intersects(*flag) {
            result.push((*name).to_string());
        }
    }
    if result.is_empty() {
        result.push("None".to_string());
    }
    result
}

// String returns a pipe-separated string of flag names.
// PORT: Go `String()` methods on flag types become `Display` impls so Go
// `%v` formatting ports to `{}`.
// Go: checker/types.go:570 TypeFlags.String

// Go: checker/types.go:574 VarianceFlags.String

// TypeAlias

// Go: checker/types.go:664 TypeAlias
#[derive(Clone, Debug, Default)]
pub struct TypeAlias {
    pub symbol: SymbolId,
    pub type_arguments: Vec<TypeId>,
}

impl TypeAlias {
    // Go: checker/types.go:669 TypeAlias.Symbol
    pub fn symbol(&self) -> SymbolId {
        self.symbol
    }

    // Go: checker/types.go:676 TypeAlias.TypeArguments
    pub fn type_arguments(&self) -> &[TypeId] {
        &self.type_arguments
    }
}

/// Go calls `(*TypeAlias).Symbol()` and `TypeArguments()` on nil aliases and
/// gets nil. `Type::alias` is `Option<Rc<TypeAlias>>`, so this trait gives the
/// same nil-safe calls: `ty.alias.symbol()`, `ty.alias.type_arguments()`.
pub trait TypeAliasExt {
    fn symbol(&self) -> SymbolId;
    fn type_arguments(&self) -> &[TypeId];
}


// Type

// PORT: Go `Type.checker` is dropped; every type lives in the arena of the
// checker that created it. Go `TypeBase` (the embedded `Type` inside each data
// struct) disappears: `Type` owns its `data`.
// PORT: layout only. `repr(C)` keeps the hot header (flags, ids, alias) in
// front of `data`, and `align(64)` starts each arena type on a cache line, so
// a flags test and the `TypeData` tag read share one line. The header is 24
// bytes and `TypeData` 104 (`TypeReference` is the largest kind), so the size
// is 128 (2 lines).
// Go: checker/types.go:685 Type

// A type must stay 2 cache lines. A rare or large field of a common kind
// goes out of line (see `ObjectType` and `StructuredType`).
// The first line holds the header and the first 40 bytes of `TypeData`: for
// a type reference, the memo (the `TypeData` tag), the type arguments, the
// target and the mapper (see `TypeReference`).




/// A value in this thread's checker arena, or a `Box` in a multi-program
/// process (`use_checker_arena`). It holds the union and intersection data
/// of `TypeData`.
///
/// PERF: a `Box` cost one malloc for each union or intersection type. The
/// arena costs a pointer bump. In a one-program process types are never
/// dropped (the checker is leaked), so the arena memory is never freed
/// either. When an arena `ArenaBox` is dropped, the value is not: its own
/// heap fields stay allocated. A released program of a multi-program
/// process frees its checkers, so there the value is a `Box`.
pub struct ArenaBox<T: 'static>(ArenaBoxRepr<T>);

enum ArenaBoxRepr<T: 'static> {
    Arena(&'static mut T),
    Owned(Box<T>),
}

impl<T> ArenaBox<T> {

}





// TypeData

// PORT: Go `TypeData` is an interface over pointers to the concrete structs
// below. Here it is an enum that owns the concrete struct. The Go interface
// methods `AsConstrainedType` ... `AsUnionOrIntersectionType` return `None`
// where Go returns nil. Go `AsType()` has no port: `Type` owns its data.
// PORT: the large variants are boxed so `Type` stays small in the arena.
// Interface, tuple, mapped, reverse mapped, evolving array and instantiation
// expression types are rare; union and intersection data is large, and in a
// one-program process it is in the checker arena (`ArenaBox`), not a malloc
// block.
// Go: checker/types.go:860 TypeData


// PORT: the arena keeps a dummy `Type` at index 0, so `TypeData` needs a
// default. It has no Go counterpart.


// IntrinsicTypeData

// Go: checker/types.go:886 IntrinsicType
#[derive(Clone, Debug, Default)]
pub struct IntrinsicType {
    pub intrinsic_name: String,
}

impl IntrinsicType {
    // Go: checker/types.go:891 IntrinsicType.IntrinsicName
    pub fn intrinsic_name(&self) -> &str {
        &self.intrinsic_name
    }
}

// LiteralTypeData

/// Go `any` value of a literal type or evaluator result:
/// `string | jsnum.Number | bool | jsnum.PseudoBigInt`. Go `nil` (a computed
/// enum value) is `Option::None` around this.
#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    String(String),
    Number(Number),
    Bool(bool),
    PseudoBigInt(PseudoBigInt),
}

// Go: checker/types.go:895 LiteralType


// PORT: Go passes a literal type's raw `value` (an `any`) as a diagnostic
// argument, and diagnostics format arguments with `%v`. This `Display` impl
// gives the same text: a string prints raw (no quotes), a number uses its
// `String()`, a bool prints true/false and a bigint prints with no `n`.
// Go: diagnostics/diagnostics.go:146 fmt.Sprintf("%v", arg)

// UniqueESSymbolTypeData

// Go: checker/types.go:920 UniqueESSymbolType
#[derive(Clone, Debug, Default)]
pub struct UniqueESSymbolType {
    pub name: String,
}

// ConstrainedType (type with computed base constraint)

// Go: checker/types.go:927 ConstrainedType
#[derive(Clone, Debug, Default)]
pub struct ConstrainedType {
    pub resolved_base_constraint: TypeId,
}

// StructuredType (base of all types with members)

// PORT: layout only. Go `signatures`, `callSignatureCount`, `indexInfos` and
// `objectTypeWithoutAbstractConstructSignatures` are in `StructuredSignatures`,
// out of line, so `Type` fits in 2 cache lines (see `Type`). Most type
// references have none of them (effect: 8.5% have signatures and 16% index
// infos), and the lists are 24 bytes each. Read them with the methods below
// and write them with `set_signatures` and
// `set_object_type_without_abstract_construct_signatures`.
// PORT: layout only. `repr(C)` keeps this field order (see `ObjectType`).
// Go: checker/types.go:936 StructuredType




// Except for tuple type references and reverse mapped types, all object types have an associated symbol.
// Possible object type instances are listed in the following.

// InterfaceType:
// ObjectFlagsClass: Originating non-generic class type
// ObjectFlagsClass|ObjectFlagsReference: Originating generic class type
// ObjectFlagsInterface: Originating non-generic interface type
// ObjectFlagsInterface|ObjectFlagsReference: Originating generic interface type

// TupleType:
// ObjectFlagsReference|ObjectFlagsTuple: Originating generic tuple type (synthesized)

// TypeReference
// ObjectFlagsReference: Instantiated generic class, interface, or tuple type

// ObjectType:
// ObjectFlagsAnonymous: Originating anonymous object type
// ObjectFlagsAnonymous|ObjectFlagsInstantiated: Instantiated anonymous object type

// MappedType:
// ObjectFlagsMapped: Originating mapped type
// ObjectFlagsMapped|ObjectFlagsInstantiated: Instantiated mapped type

// InstantiationExpressionType:
// ObjectFlagsAnonymous|ObjectFlagsInstantiationExpression: Originating instantiation expression type
// ObjectFlagsAnonymous|ObjectFlagsInstantiated|ObjectFlagsInstantiationExpression: Instantiated instantiation expression type

// ReverseMappedType:
// ObjectFlagsAnonymous|ObjectFlagsReverseMapped: Reverse mapped type

// EvolvingArrayType:
// ObjectFlagsEvolvingArray: Evolving array type


// PORT: Go `instantiations` (the map of type instantiations) is not here.
// Few object types have one (effect: 7.6k anonymous object types of 129k,
// 95 type references of 165k), so the 32-byte map would grow every `Type`.
// An interface or tuple keeps it in `InterfaceType`; any other object type
// has it in `Checker::object_type_instantiations`, at the index in
// `instantiations`. Read and write it with `Checker::object_instantiations`
// and `object_instantiations_mut`.
// PORT: layout only. `repr(C)` keeps `target` and `mapper` first. An
// anonymous object type then has them, its members table and its
// signatures on the first cache line of its `Type` (see `TypeReference`).
// Go: checker/types.go:994 ObjectType

/// The index of an object type's instantiation map in
/// `Checker::object_type_instantiations`. `NIL` is a Go nil map, and it is
/// always `NIL` on an interface or tuple (see `ObjectType`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InstantiationMapId(pub u32);

impl InstantiationMapId {
    pub const NIL: Self = Self(0);
}

// TypeReference (instantiation of an InterfaceType)

// PORT: layout only. `repr(C)` keeps this field order, so the first cache
// line of a type reference's `Type` holds the `TypeData` tag (the memo, see
// `GenericArgumentsMemo`), the type arguments, the target and the mapper.
// The resolved members and `node` are on the second line.
// Go: checker/types.go:1005 TypeReference

/// Memo state of `Checker::is_type_reference_with_generic_arguments`.
///
/// PORT: layout only. An enum (not an integer) leaves invalid values that
/// `TypeData` uses for its tag, so the tag does not grow `Type`. It is
/// `u64`: the tags of `SharedList` and `Option<ArenaBox>` are 8 bytes wide
/// too, and of fields with the same number of invalid values the compiler
/// takes the first one. So the `TypeData` tag is this field, on the first
/// cache line, and the other kinds fit after it (with a `u32` memo the
/// compiler took a list tag on the second line, and `TypeData` grew by an
/// 8-byte tag). The field has 8 bytes of room anyway: the list after it is
/// 8-byte aligned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u64)]
pub enum GenericArgumentsMemo {
    #[default]
    Unknown = 0,
    False = 1,
    True = 2,
}

// InterfaceType (when generic, serves as reference to instantiation of itself)

// Go: checker/types.go:1015 InterfaceType


// TupleType

// Go: checker/types.go:1073 TupleElementInfo
#[derive(Clone, Copy, Debug, Default)]
pub struct TupleElementInfo {
    pub flags: ElementFlags,
    pub labeled_declaration: Node, // NamedTupleMember | ParameterDeclaration | nil
}

impl TupleElementInfo {
    // Go: checker/types.go:1078 TupleElementInfo.TupleElementFlags
    pub fn tuple_element_flags(&self) -> ElementFlags {
        self.flags
    }

    // Go: checker/types.go:1079 TupleElementInfo.LabeledDeclaration
    pub fn labeled_declaration(&self) -> Node {
        self.labeled_declaration
    }
}

// Go: checker/types.go:1081 TupleType


// InstantiationExpressionType

// Go: checker/types.go:1103 InstantiationExpressionType

// MappedType

// Go: checker/types.go:1110 MappedType


// ReverseMappedType

// Go: checker/types.go:1135 ReverseMappedType

// EvolvingArrayType

// Go: checker/types.go:1144 EvolvingArrayType

// UnionOrIntersectionTypeData

// Go: checker/types.go:1152 UnionOrIntersectionType


// UnionType

// Go: checker/types.go:1168 UnionType

// IntersectionType

// Go: checker/types.go:1179 IntersectionType

// TypeParameter

// Go: checker/types.go:1187 TypeParameter
#[derive(Clone, Debug, Default)]
pub struct TypeParameter {
    pub constrained: ConstrainedType,
    pub constraint: TypeId,
    pub target: TypeId,
    pub mapper: MapperId,
    pub is_this_type: bool,
    pub is_distributed: bool,
    pub resolved_default_type: TypeId,
    pub distributed_type: TypeId,
}

impl TypeParameter {
    // Go: checker/types.go:1198 TypeParameter.IsThisType
    pub fn is_this_type(&self) -> bool {
        self.is_this_type
    }
}

// IndexType

// Go: checker/types.go:1213 IndexType
#[derive(Clone, Debug, Default)]
pub struct IndexType {
    pub constrained: ConstrainedType,
    pub target: TypeId,
    pub index_flags: IndexFlags,
}

impl IndexType {
    // Go: checker/types.go:1219 IndexType.Target
    #[inline]
    pub fn target(&self) -> TypeId {
        self.target
    }
}

// IndexedAccessType

// Go: checker/types.go:1223 IndexedAccessType
#[derive(Clone, Debug, Default)]
pub struct IndexedAccessType {
    pub constrained: ConstrainedType,
    pub object_type: TypeId,
    pub index_type: TypeId,
    pub access_flags: AccessFlags, // Only includes AccessFlags.Persistent
}

impl IndexedAccessType {
    // Go: checker/types.go:1230 IndexedAccessType.ObjectType
    pub fn object_type(&self) -> TypeId {
        self.object_type
    }

    // Go: checker/types.go:1231 IndexedAccessType.IndexType
    pub fn index_type(&self) -> TypeId {
        self.index_type
    }
}

// PORT: Go shares the `texts` and `types` slices; `Rc<[_]>` makes the
// clones cheap. The contents never change after creation.
// Go: checker/types.go:1233 TemplateLiteralType


// Go: checker/types.go:1242 StringMappingType
#[derive(Clone, Debug, Default)]
pub struct StringMappingType {
    pub constrained: ConstrainedType,
    pub target: TypeId,
}

impl StringMappingType {
    // Go: checker/types.go:1247 StringMappingType.Target
    #[inline]
    pub fn target(&self) -> TypeId {
        self.target
    }
}

// Go: checker/types.go:1249 SubstitutionType
#[derive(Clone, Debug, Default)]
pub struct SubstitutionType {
    pub constrained: ConstrainedType,
    pub base_type: TypeId,  // Target type
    pub constraint: TypeId, // Constraint that target type is known to satisfy
}

impl SubstitutionType {
    // Go: checker/types.go:1255 SubstitutionType.BaseType
    pub fn base_type(&self) -> TypeId {
        self.base_type
    }

    // Go: checker/types.go:1256 SubstitutionType.SubstConstraint
    pub fn subst_constraint(&self) -> TypeId {
        self.constraint
    }
}

// PORT: Go shares one `*ConditionalRoot` between a conditional type and all
// its instantiations and mutates `instantiations`, so `ConditionalType.root`
// is `Rc<RefCell<ConditionalRoot>>`.
// Go: checker/types.go:1258 ConditionalRoot

// Go: checker/types.go:1269 ConditionalType


// Signature

// Go: checker/types.go:1312 Signature
#[derive(Clone, Debug, Default)]
pub struct Signature {
    pub id: SignatureId,
    pub flags: SignatureFlags,
    pub min_argument_count: i32,
    pub resolved_min_argument_count: i32,
    pub declaration: Node,
    pub type_parameters: Vec<TypeId>,
    // PORT: Go compares signature type parameter lists by slice identity
    // (`core.Same` in compareSignaturesRelated). A Rust signature owns its
    // `Vec`, so a list that Go shares between signatures (cloneSignature,
    // class local type parameters, inferred type parameters, ...) carries
    // the same nonzero origin here. 0 means the list belongs only to this
    // signature. See `Checker::same_signature_type_parameters`.
    pub type_parameters_origin: u32,
    pub parameters: Vec<SymbolId>,
    pub this_parameter: SymbolId,
    pub resolved_return_type: TypeId,
    pub resolved_type_predicate: TypePredicateId,
    pub target: SignatureId,
    pub mapper: MapperId,
    pub isolated_signature_type: TypeId,
    // PORT: Go `*CompositeSignature` is created once and only read; nil is `None`.
    pub composite: Option<Rc<CompositeSignature>>,
}

impl Signature {
    // Go: checker/types.go:1329 Signature.Id
    pub fn id(&self) -> SignatureId {
        self.id
    }

    // Go: checker/types.go:1333 Signature.Flags
    pub fn flags(&self) -> SignatureFlags {
        self.flags
    }

    // Go: checker/types.go:1337 Signature.TypeParameters
    pub fn type_parameters(&self) -> &[TypeId] {
        &self.type_parameters
    }

    // Go: checker/types.go:1341 Signature.Declaration
    pub fn declaration(&self) -> Node {
        self.declaration
    }

    // Go: checker/types.go:1345 Signature.Target
    #[inline]
    pub fn target(&self) -> SignatureId {
        self.target
    }

    // Go: checker/types.go:1349 Signature.ThisParameter
    pub fn this_parameter(&self) -> SymbolId {
        self.this_parameter
    }

    // Go: checker/types.go:1353 Signature.Parameters
    pub fn parameters(&self) -> &[SymbolId] {
        &self.parameters
    }

    // Go: checker/types.go:1357 Signature.HasRestParameter
    pub fn has_rest_parameter(&self) -> bool {
        self.flags.intersects(SignatureFlags::HAS_REST_PARAMETER)
    }

    // Go: checker/types.go:1361 Signature.MinArgumentCount
    pub fn min_argument_count(&self) -> i32 {
        self.min_argument_count
    }
}

// Go: checker/types.go:1365 CompositeSignature
#[derive(Clone, Debug, Default)]
pub struct CompositeSignature {
    pub is_union: bool,               // True for union, false for intersection
    pub signatures: Vec<SignatureId>, // Individual signatures
}

// Go: checker/types.go:1379 TypePredicate
#[derive(Clone, Debug, Default)]
pub struct TypePredicate {
    pub kind: TypePredicateKind,
    pub parameter_index: i32,
    pub parameter_name: String,
    // PORT: Go field `t`.
    pub t: TypeId,
}

impl TypePredicate {
    // Go: checker/types.go:1386 TypePredicate.Type
    pub fn type_(&self) -> TypeId {
        self.t
    }

    // Go: checker/types.go:1390 TypePredicate.Kind
    pub fn kind(&self) -> TypePredicateKind {
        self.kind
    }

    // Go: checker/types.go:1394 TypePredicate.ParameterIndex
    pub fn parameter_index(&self) -> i32 {
        self.parameter_index
    }

    // Go: checker/types.go:1398 TypePredicate.ParameterName
    pub fn parameter_name(&self) -> &str {
        &self.parameter_name
    }
}

// IndexInfo

// Go: checker/types.go:1404 IndexInfo
#[derive(Clone, Debug, Default)]
pub struct IndexInfo {
    pub key_type: TypeId,
    pub value_type: TypeId,
    pub is_readonly: bool,
    pub declaration: Node,      // IndexSignatureDeclaration
    pub index_symbol: SymbolId, // Synthetic property symbol for this index signature
    pub components: Vec<Node>,  // ElementWithComputedPropertyName
}

impl IndexInfo {
    // Go: checker/types.go:1413 IndexInfo.KeyType
    pub fn key_type(&self) -> TypeId {
        self.key_type
    }

    // Go: checker/types.go:1417 IndexInfo.ValueType
    pub fn value_type(&self) -> TypeId {
        self.value_type
    }

    // Go: checker/types.go:1421 IndexInfo.IsReadonly
    pub fn is_readonly(&self) -> bool {
        self.is_readonly
    }

    // Go: checker/types.go:1425 IndexInfo.Declaration
    pub fn declaration(&self) -> Node {
        self.declaration
    }
}

/*
 * Ternary values are defined such that
 * x & y picks the lesser in the order False < Unknown < Maybe < True, and
 * x | y picks the greater in the order False < Unknown < Maybe < True.
 * Generally, Ternary.Maybe is used as the result of a relation that depends on itself, and
 * Ternary.Unknown is used as the result of a variance check that depends on itself. We make
 * a distinction because we don't want to cache circular variance check results.
 */
// PORT: `Ternary` itself is generated in `crate::flags` (a `go_enum!`).
// These operators give Go's `x & y` and `x | y` on the underlying int8.
impl std::ops::BitAnd for Ternary {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAndAssign for Ternary {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl std::ops::BitOr for Ternary {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Ternary {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

// PORT: Go `TypeComparer` is a func value stored in checker fields and
// inference contexts, so it is an `Rc<dyn Fn>` that gets the checker first.
// Go: checker/types.go:1446 TypeComparer

// Go: checker/types.go:1448 LanguageFeatureMinimumTargetMap
#[derive(Clone, Copy, Debug)]
pub struct LanguageFeatureMinimumTargetMap {
    pub exponentiation: ScriptTarget,
    pub async_functions: ScriptTarget,
    pub for_await_of: ScriptTarget,
    pub async_generators: ScriptTarget,
    pub async_iteration: ScriptTarget,
    pub object_spread_rest: ScriptTarget,
    pub regular_expression_flags_dot_all: ScriptTarget,
    pub bindingless_catch: ScriptTarget,
    pub big_int: ScriptTarget,
    pub nullish_coalesce: ScriptTarget,
    pub optional_chaining: ScriptTarget,
    pub logical_assignment: ScriptTarget,
    pub top_level_await: ScriptTarget,
    pub class_fields: ScriptTarget,
    pub private_names_and_class_static_blocks: ScriptTarget,
    pub regular_expression_flags_has_indices: ScriptTarget,
    pub shebang_comments: ScriptTarget,
    pub using_and_await_using: ScriptTarget,
    pub class_and_class_element_decorators: ScriptTarget,
    pub regular_expression_flags_unicode_sets: ScriptTarget,
}

// PORT: Go package var `LanguageFeatureMinimumTarget` is never mutated, so
// it is a Rust const.
// Go: checker/types.go:1471 LanguageFeatureMinimumTarget
pub const LANGUAGE_FEATURE_MINIMUM_TARGET: LanguageFeatureMinimumTargetMap =
    LanguageFeatureMinimumTargetMap {
        exponentiation: ScriptTarget::ES2016,
        async_functions: ScriptTarget::ES2017,
        for_await_of: ScriptTarget::ES2018,
        async_generators: ScriptTarget::ES2018,
        async_iteration: ScriptTarget::ES2018,
        object_spread_rest: ScriptTarget::ES2018,
        regular_expression_flags_dot_all: ScriptTarget::ES2018,
        bindingless_catch: ScriptTarget::ES2019,
        big_int: ScriptTarget::ES2020,
        nullish_coalesce: ScriptTarget::ES2020,
        optional_chaining: ScriptTarget::ES2020,
        logical_assignment: ScriptTarget::ES2021,
        top_level_await: ScriptTarget::ES2022,
        class_fields: ScriptTarget::ES2022,
        private_names_and_class_static_blocks: ScriptTarget::ES2022,
        regular_expression_flags_has_indices: ScriptTarget::ES2022,
        shebang_comments: ScriptTarget::ES_NEXT,
        using_and_await_using: ScriptTarget::ES_NEXT,
        class_and_class_element_decorators: ScriptTarget::ES_NEXT,
        regular_expression_flags_unicode_sets: ScriptTarget::ES_NEXT,
    };

// Aliases for types
// Go: checker/types.go:1495 StringLiteralType

// PORT: no Go counterpart. Go allocates each `Type` and `TypeMapper` on its
// own; the port keeps them in index arenas. A plain `Vec` arena doubles and
// copies every entry when it grows, and keeps up to half its capacity unused.
// `ChunkedArena` stores entries in chunks of `ARENA_CHUNK_LEN`, so growing it
// never moves an entry of a later chunk. Index 0 is the nil dummy, like the
// other arenas.
//
// PERF (rss2): the first chunk starts at `ARENA_FIRST_CAPACITY` entries. It
// doubles up to `ARENA_CHUNK_STEP` entries, then grows in steps of as many
// until it is full. A first chunk made at full size was mostly allocated but
// never written in a small program (query: about 5,300 of 8,192 `Type` slots
// in each checker), and jemalloc gives the heap huge pages, which make such
// memory resident. Later chunks are made at full size: a program that fills
// the first chunk is large, and growing each chunk too cost about 1.5% of
// check time on hono, zod and effect (rss2 grid: copies and page faults).
pub struct ChunkedArena<T> {
    chunks: Vec<Vec<T>>,
    len: usize,
}

// PERF: 8,192 entries a chunk. Every `ty(t)` reads the chunk table before the
// entry, so a smaller table stays in L1 more (elysia: 720 chunks, 17 KB, was
// 1,440). One chunk of each element type stays under 2 MiB, so a small project
// does not get a 2 MiB huge page for each arena. `Type` is 128 bytes, so 13 is
// the largest shift under that limit.
const ARENA_CHUNK_SHIFT: usize = 13;
const ARENA_CHUNK_LEN: usize = 1 << ARENA_CHUNK_SHIFT;
const ARENA_CHUNK_MASK: usize = ARENA_CHUNK_LEN - 1;
const ARENA_CHUNK_MAX_BYTES: usize = 2 << 20;
/// Each step of the first chunk after it has this many entries adds as many.
const ARENA_CHUNK_STEP: usize = ARENA_CHUNK_LEN / 4;
/// The room of the first chunk when the arena is made.
const ARENA_FIRST_CAPACITY: usize = 64;

impl<T> ChunkedArena<T> {
    /// Creates an arena that holds only `nil`, the dummy entry at index 0.
    pub fn with_nil(nil: T) -> Self {
        let mut arena = ChunkedArena {
            chunks: vec![Vec::with_capacity(ARENA_FIRST_CAPACITY)],
            len: 1,
        };
        arena.chunks[0].push(nil);
        arena
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // PERF: always inlined, so the caller builds `value` straight in its
    // chunk slot. Out of line, the caller wrote the value to the stack with
    // small stores and this function read it back at once with 16-byte
    // loads (a store forwarding stall), or copied a `Type` with `memmove`.
    // The rare new-chunk path stays out of line.
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        self.push_with(move || value);
    }

    /// Pushes the value `make` returns. `make` runs after the chunk slot is
    /// ready, so a large value (a `Type`) is built straight in its slot.
    ///
    /// PERF: with `push(make())` the value is built first, and the grow
    /// branch of `Vec::push` then sits between building and storing, so the
    /// value goes to the stack and is copied with `memcpy`. Here the room
    /// tests come first. The second one never fails: a full first chunk gets
    /// its next step (`grow_chunk`), a later chunk is made with room for
    /// `ARENA_CHUNK_LEN` entries, and the entry after a full chunk gets a new
    /// chunk. When `make` makes no call, LLVM knows the chunk did not change
    /// after the second test and drops the grow branch of the `Vec::push`
    /// below. When the first test fails (no step), the second one is known to
    /// fail too, so the hot path makes one test, as before.
    #[inline(always)]
    pub fn push_with(&mut self, make: impl FnOnce() -> T) {
        if self.len & ARENA_CHUNK_MASK == 0 {
            self.add_chunk();
        }
        let chunk = self.chunks.last_mut().expect("arena chunk");
        if chunk.len() == chunk.capacity() {
            grow_chunk(chunk);
        }
        if chunk.len() == chunk.capacity() {
            arena_chunk_full();
        }
        chunk.push(make());
        self.len += 1;
    }

    /// Adds the empty chunk for the next `ARENA_CHUNK_LEN` entries.
    #[cold]
    #[inline(never)]
    fn add_chunk(&mut self) {
        self.chunks.push(Vec::with_capacity(ARENA_CHUNK_LEN));
    }
}

/// Gives the full first chunk room for its next `ARENA_CHUNK_STEP` entries,
/// or doubles it while it is smaller than that. Later chunks are made full
/// size and never get here.
#[cold]
#[inline(never)]
fn grow_chunk<T>(chunk: &mut Vec<T>) {
    chunk.reserve_exact(chunk.len().min(ARENA_CHUNK_STEP));
}

/// The last arena chunk has no room after `grow_chunk`. `push_with` never
/// gets here.
#[cold]
#[inline(never)]
fn arena_chunk_full() -> ! {
    panic!("arena chunk is full")
}

impl<T> std::ops::Index<usize> for ChunkedArena<T> {
    type Output = T;

    #[inline]
    fn index(&self, i: usize) -> &T {
        &self.chunks[i >> ARENA_CHUNK_SHIFT][i & ARENA_CHUNK_MASK]
    }
}

impl<T> std::ops::IndexMut<usize> for ChunkedArena<T> {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.chunks[i >> ARENA_CHUNK_SHIFT][i & ARENA_CHUNK_MASK]
    }
}
