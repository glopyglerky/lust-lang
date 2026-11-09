//! Port of `checker/checker.go` lines 1-1115: checker-wide key and state
//! types, `Checker` itself (with the arenas from `PORTING.md`), and
//! `NewChecker` as `Checker::new`.
//!
//! PORT: Go consts in this range (`CheckMode`, `TypeSystemPropertyName`,
//! `WideningKind`, `CachedTypeKind`, `InferenceFlags`, `InferencePriority`,
//! `DeclarationMeaning`, `DeclarationSpaces`, `IntrinsicTypeKind`,
//! `MappedTypeModifiers`, `MappedTypeNameTypeKind`, `ReferenceHint`,
//! `TypeFacts`, `IterationUse`, `IterationTypeKind`) are generated in
//! `crate::flags` and are not repeated here.

use crate::gostd::Context;
use crate::jsnum::{Number, PseudoBigInt};
use crate::prelude::*;
use std::sync::LazyLock;

// Go: checker/checker.go:52 TypeSystemEntity
// PORT: Go `TypeSystemEntity any`. The Go callers of `pushTypeResolution`
// pass a declaration node, a symbol, a type or a signature, so the Rust type
// is an enum over those four handles. Go compares targets with `==` on the
// interface value, which is handle equality here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TypeSystemEntity {
    Node(Node),
    Symbol(SymbolId),
    Type(TypeId),
    Signature(SignatureId),
}

impl From<Node> for TypeSystemEntity {
    fn from(node: Node) -> Self {
        TypeSystemEntity::Node(node)
    }
}

impl From<SymbolId> for TypeSystemEntity {
    fn from(symbol: SymbolId) -> Self {
        TypeSystemEntity::Symbol(symbol)
    }
}

impl From<TypeId> for TypeSystemEntity {
    fn from(t: TypeId) -> Self {
        TypeSystemEntity::Type(t)
    }
}

impl From<SignatureId> for TypeSystemEntity {
    fn from(sig: SignatureId) -> Self {
        TypeSystemEntity::Signature(sig)
    }
}

// Go: checker/checker.go:69 TypeResolution
#[derive(Clone, Copy, Debug)]
pub struct TypeResolution {
    pub target: TypeSystemEntity,
    pub property_name: TypeSystemPropertyName,
    pub result: bool,
}

// ContextualInfo

// Go: checker/checker.go:77 ContextualInfo
#[derive(Clone, Copy, Debug, Default)]
pub struct ContextualInfo {
    pub node: Node,
    pub t: TypeId,
    pub is_cache: bool,
}

// InferenceContextInfo

// Go: checker/checker.go:85 InferenceContextInfo
#[derive(Clone, Copy, Debug, Default)]
pub struct InferenceContextInfo {
    pub node: Node,
    pub context: InferenceContextId,
}

// EnumLiteralKey

// Go: checker/checker.go:103 EnumLiteralKey
// PORT: Go `value any` holds a `LiteralValue`. `LiteralValue` is not `Hash`
// (it holds `f64`), so the key stores `LiteralValueKey`, which compares like
// Go interface `==` (numbers as floats, -0 == +0). Build it with
// `LiteralValueKey::from(&value)`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EnumLiteralKey {
    pub enum_symbol: SymbolId,
    pub value: LiteralValueKey,
}

// PORT: hashable form of `LiteralValue` for Go map keys of type `any`.
// NaN cannot match in Go; Go callers keep NaN out of these keys.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum LiteralValueKey {
    String(String),
    Number(NumberKey),
    Bool(bool),
    PseudoBigInt(PseudoBigIntKey),
}

impl From<&LiteralValue> for LiteralValueKey {
    fn from(v: &LiteralValue) -> Self {
        match v {
            LiteralValue::String(s) => LiteralValueKey::String(s.clone()),
            LiteralValue::Number(n) => LiteralValueKey::Number(NumberKey::from(Number(n.0))),
            LiteralValue::Bool(b) => LiteralValueKey::Bool(*b),
            LiteralValue::PseudoBigInt(p) => {
                LiteralValueKey::PseudoBigInt(PseudoBigIntKey::from(p))
            }
        }
    }
}

// EnumRelationKey

// Go: checker/checker.go:110 EnumRelationKey
// PORT: Go `ast.SymbolId` values are symbol handles here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EnumRelationKey {
    pub source_id: SymbolId,
    pub target_id: SymbolId,
}

// CachedTypeKey

// Go: checker/checker.go:146 CachedTypeKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CachedTypeKey {
    pub kind: CachedTypeKind,
    pub type_id: TypeId,
}

// NarrowedTypeKey

// Go: checker/checker.go:153 NarrowedTypeKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NarrowedTypeKey {
    pub t: TypeId,
    pub candidate: TypeId,
    pub assume_true: bool,
    pub check_derived: bool,
}

// UnionOfUnionKey

// Go: checker/checker.go:162 UnionOfUnionKey

// CachedSignatureKey

// Go: checker/checker.go:171 CachedSignatureKey

// PORT: Go `CacheHashKey(xxh3.HashString128(s))`. A streaming xxh3 hash of
// the same bytes equals the one-shot hash, so this goes through the Go
// `keyBuilder` port (`KeyBuilder`) that owns the hashing.

// Go: checker/checker.go:171 SignatureKeyErased
// PORT: Go package vars become lazily computed statics. Read with
// `*SIGNATURE_KEY_ERASED`.
// Go: checker/checker.go:172 SignatureKeyCanonical
// Go: checker/checker.go:173 SignatureKeyBase
// Go: checker/checker.go:174 SignatureKeyInner
// Go: checker/checker.go:175 SignatureKeyOuter

// StringMappingKey

// Go: checker/checker.go:186 StringMappingKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct StringMappingKey {
    pub s: SymbolId,
    pub t: TypeId,
}

// AssignmentReducedKey

// Go: checker/checker.go:193 AssignmentReducedKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct AssignmentReducedKey {
    pub id1: TypeId,
    pub id2: TypeId,
}

// DiscriminatedContextualTypeKey

// Go: checker/checker.go:200 DiscriminatedContextualTypeKey
// PORT: Go `ast.NodeId` is the node handle here (a `Node` is unique per
// node, like a Go node id).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct DiscriminatedContextualTypeKey {
    pub node_id: Node,
    pub type_id: TypeId,
}

// InstantiationExpressionKey

// Go: checker/checker.go:207 InstantiationExpressionKey
// PORT: Go `ast.NodeId` is the node handle here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InstantiationExpressionKey {
    pub node_id: Node,
    pub type_id: TypeId,
}

// SubstitutionTypeKey

// Go: checker/checker.go:214 SubstitutionTypeKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SubstitutionTypeKey {
    pub base_id: TypeId,
    pub constraint_id: TypeId,
}

// ReverseMappedTypeKey

// Go: checker/checker.go:221 ReverseMappedTypeKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ReverseMappedTypeKey {
    pub source_id: TypeId,
    pub target_id: TypeId,
    pub constraint_id: TypeId,
}

// IterationTypesKey

// Go: checker/checker.go:229 IterationTypesKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IterationTypesKey {
    pub type_id: TypeId,
    pub use_: IterationUse,
}

// PropertiesTypesKey

// Go: checker/checker.go:236 PropertiesTypesKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PropertiesTypesKey {
    pub type_id: TypeId,
    pub include: TypeFlags,
    pub include_origin: bool,
}

// NonExistentPropertyKey

// Go: checker/checker.go:244 NonExistentPropertyKey
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NonExistentPropertyKey {
    pub prop_node: Node,
    pub containing_type: TypeId,
    pub is_unchecked_js: bool,
}

// FlowLoopKey

// Go: checker/checker.go:252 FlowLoopKey

// Go: checker/checker.go:257 FlowLoopInfo

// InferenceContext

// Go: checker/checker.go:276 InferenceContext
// PERF (infermem1): 56 bytes (asserted below), not 120. The port keeps every context
// of a run (typebox: about 3.9M, almost all from conditional types with
// `infer`), so the fields that those contexts never set are in `rare`, a box
// that is made on the first write. The accessors give Go's zero values while
// it is absent.

// 32-bit targets (wasm32) have smaller pointers and do not check this.

/// The fields of Go `InferenceContext` that only signature inference sets.
#[derive(Clone, Default)]
pub struct InferenceContextRare {
    /// Type mapper for inferences from return types (if any)
    pub return_mapper: MapperId,
    /// Type mapper for inferences from return types of outer function (if any)
    pub outer_return_mapper: MapperId,
    /// Inferred type parameters for function result
    pub inferred_type_parameters: Vec<TypeId>,
    // PORT: slice identity of `inferred_type_parameters` (see
    // `Signature::type_parameters_origin`); 0 while the list is empty.
    pub inferred_type_parameters_origin: u32,
    pub intra_expression_inference_sites: Vec<IntraExpressionInferenceSite>,
}


// PORT: `Default` exists only for the dummy entry at index 0 of
// `Checker::inference_contexts`. Go never has a nil `compareTypes`, so the
// placeholder comparer panics like a call of a nil Go func.

// Go: checker/checker.go:289 InferenceInfo
// PORT: Go `*InferenceInfo` is an index into `InferenceContext::inferences`.
// PERF (infermem1): 32 bytes (asserted below), not 72. The two candidate
// lists are in one box that the first candidate makes (typebox: about 6M
// infos, under a third of them with candidates). `candidates()` and
// `contra_candidates()` give Go's nil lists while it is absent.
#[derive(Clone, Debug, Default)]
pub struct InferenceInfo {
    /// Type parameter for which inferences are being made
    pub type_parameter: TypeId,
    /// Go `candidates` and `contraCandidates`, or `None` while both are nil.
    pub candidate_lists: Option<Box<InferenceCandidates>>,
    /// Cache for resolved inferred type
    pub inferred_type: TypeId,
    /// Priority of current inference set
    pub priority: InferencePriority,
    /// True if all inferences are to top level occurrences
    pub top_level: bool,
    /// True if inferences are fixed
    pub is_fixed: bool,
    /// Implied arity (or -1)
    pub implied_arity: i32,
}

// 32-bit targets (wasm32) have smaller pointers and do not check this.
#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::size_of::<InferenceInfo>() == 32);

/// The candidate lists of an `InferenceInfo`.
#[derive(Clone, Debug, Default)]
pub struct InferenceCandidates {
    /// Candidates in covariant positions in decreasing depth order
    pub candidates: Vec<TypeId>,
    /// Candidates in contravariant positions
    pub contra_candidates: Vec<TypeId>,
}

impl InferenceInfo {
    /// Go `info.candidates`.
    #[must_use]
    pub fn candidates(&self) -> &[TypeId] {
        self.candidate_lists.as_ref().map_or(&[], |l| &l.candidates)
    }

    /// Go `info.contraCandidates`.
    #[must_use]
    pub fn contra_candidates(&self) -> &[TypeId] {
        self.candidate_lists
            .as_ref()
            .map_or(&[], |l| &l.contra_candidates)
    }

    /// The candidate lists for a write. Makes the box on the first write.
    pub fn candidate_lists_mut(&mut self) -> &mut InferenceCandidates {
        self.candidate_lists.get_or_insert_default()
    }
}

// Go: checker/checker.go:321 IntraExpressionInferenceSite
#[derive(Clone, Copy, Debug, Default)]
pub struct IntraExpressionInferenceSite {
    pub node: Node,
    pub t: TypeId,
}

// Go: checker/checker.go:360 intrinsicTypeKinds
// PORT: Go package map var; read with `INTRINSIC_TYPE_KINDS.get(name)`.
pub static INTRINSIC_TYPE_KINDS: LazyLock<FxHashMap<&'static str, IntrinsicTypeKind>> =
    LazyLock::new(|| {
        let mut m = FxHashMap::default();
        m.insert("Uppercase", IntrinsicTypeKind::UPPERCASE);
        m.insert("Lowercase", IntrinsicTypeKind::LOWERCASE);
        m.insert("Capitalize", IntrinsicTypeKind::CAPITALIZE);
        m.insert("Uncapitalize", IntrinsicTypeKind::UNCAPITALIZE);
        m.insert("NoInfer", IntrinsicTypeKind::NO_INFER);
        m
    });

// Go: checker/checker.go:507 IterationTypes
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct IterationTypes {
    pub yield_type: TypeId,
    pub return_type: TypeId,
    pub next_type: TypeId,
}

// Go: checker/checker.go:521 IterationTypesResolver
// PORT: Go func fields are `Rc<dyn Fn(&mut Checker, ...)>`. Go
// `*diagnostics.Message` fields are always set by
// `initializeIterationResolvers`, so they are plain `&'static Message`.

// Go: checker/checker.go:537 WideningContext
// PORT: Go `*WideningContext` links become `Rc<RefCell<WideningContext>>`;
// a nil parent is `None`.
#[derive(Clone, Default)]
pub struct WideningContext {
    /// Parent context
    // PORT: weak, so a parent and its `child_contexts` do not form an `Rc`
    // cycle that leaks (Go's GC frees them). A child is only used while its
    // parent is alive: the widening recursion holds the parent.
    pub parent: Option<std::rc::Weak<RefCell<WideningContext>>>,
    /// Name of property in parent
    pub property_name: String,
    /// Types of siblings
    pub siblings: Vec<TypeId>,
    /// Properties occurring in sibling object literals
    pub resolved_properties: Vec<SymbolId>,
    pub child_contexts: FxHashMap<String, Rc<RefCell<WideningContext>>>,
    pub widened_types: FxHashMap<TypeId, TypeId>,
}

// Go: checker/checker.go:546 VarianceStackEntry
#[derive(Clone, Debug, Default)]
pub struct VarianceStackEntry {
    pub symbol: SymbolId,
    pub type_parameters: Vec<TypeId>,
}

// Go: checker/checker.go:551 maxSerializationLevel
pub const MAX_SERIALIZATION_LEVEL: i32 = 2;

// PORT: Go `Program` and `Host` interfaces (checker.go:555-583) are not
// ported as types. The checker reads the installed program through
// `prog()` and the free functions in `program.rs`.

// PORT: Go `map[jsnum.Number]*Type` key. `crate::jsnum::Number` is not `Hash`.
// Go compares float map keys with `==`, so -0 and +0 are one key. NaN never
// matches in Go; `getNumberLiteralType` caches NaN separately, so NaN never
// reaches this key.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct NumberKey(pub u64);

impl From<Number> for NumberKey {
    fn from(n: Number) -> Self {
        let v = if n.0 == 0.0 { 0.0 } else { n.0 };
        NumberKey(v.to_bits())
    }
}

// PORT: Go `map[jsnum.PseudoBigInt]*Type` key. `crate::jsnum::PseudoBigInt` is
// not `Hash`. Go compares the struct fields, like this key.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PseudoBigIntKey {
    pub negative: bool,
    pub base10_value: String,
}

impl From<&PseudoBigInt> for PseudoBigIntKey {
    fn from(v: &PseudoBigInt) -> Self {
        PseudoBigIntKey {
            negative: v.negative,
            base10_value: v.base10_value.clone(),
        }
    }
}

impl From<PseudoBigInt> for PseudoBigIntKey {
    fn from(v: PseudoBigInt) -> Self {
        PseudoBigIntKey {
            negative: v.negative,
            base10_value: v.base10_value,
        }
    }
}

// PORT: Go `symbolTableID` (symbolaccessibility.go:402) is used as a map key
// in a `Checker` field, so its Rust type name is fixed here. The owner of
// symbolaccessibility.go defines its constants and constructors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolTableID(pub u64);

// PORT: type aliases for the Go func-typed `Checker` fields. They are plain
// aliases, so code that spells out the `Rc<dyn Fn ...>` type still matches.

// PORT: placeholders for Go func fields before `NewChecker` assigns them.
// Calling one panics, like calling a nil Go func.









// PORT: stands for a nil `*IterationTypesResolver`. Every func field panics.
// The message fields need some `&'static Message`; Go never reads them from
// a nil resolver, so the values are never observed.



// PORT: perf. `FlatMap` indexes by the low bits of the hash, and type ids
// are dense, so the id is multiplied by the 64-bit golden ratio and the high
// half is folded down to mix the low bits. The nil id is the empty-slot
// marker; `FlatMap` keeps that key in its side slot.
impl FlatKey for TypeId {
    #[inline]
    fn flat_hash(&self) -> u64 {
        let x = u64::from(self.0).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        x ^ (x >> 32)
    }
}

// Checker

// Go: checker/checker.go:585 Checker
// PORT: Go fields keep their order and snake names. Differences:
// - `program Program` is `&'static GoProgram` (the installed `prog()`).
// - `symbolArena`, `signatureArena`, `indexInfoArena` are replaced by the
//   arenas at the end (`symbols`, `types`, `signatures`, `index_infos`,
//   `type_predicates`, `mappers`, `inference_contexts`), see `PORTING.md`.
// - `regExpScanner` (Go scanner) and `mu` are out of scope (scanner object,
//   concurrency) and are not fields. The nil-able Go `ctx` is
//   `Option<Context>`. `tracer` is the last field (see `crate::tracing`).
//   `emitResolver` plus `emitResolverOnce` is the `Option<Rc<EmitResolver>>`
//   field `emit_resolver`.
// - `sync.Once` fields become `bool` "done" flags.
// - `*T` pools and shared structs (`*Relation`, `*Relater`, `*FlowState`,
//   `*InferenceState`) are `Rc<RefCell<T>>`; nil-able ones are `Option`.
// - `ast.NodeId` map keys are `Node` handles.

// Arena accessors (PORTING.md "Checker data"). Handles index their arena
// directly; index 0 is the nil dummy.

// Go: checker/checker.go:911 NewChecker
// PORT: Go `NewChecker(program) (*Checker, *sync.Mutex)` becomes
// `Checker::new(checker_index)`. The program is the installed `prog()`; the
// mutex is dropped, and the tracer comes from the process tracing session
// (`crate::tracing::new_checker_tracer`). Go `c.id = nextCheckerID.Add(1)` numbers
// checkers from 1 in creation order; the pool creates them in index order,
// so `id = checker_index + 1`.
// PORT: Go binds every file (`program.BindSourceFiles`) before it creates
// checkers. Here the checker binds the program first if nothing has
// (`program::bind_all`), then copies the program's binder symbols
// (`program::bound_symbols`) as its own symbol arena
// (`SymbolArena::for_checker`). `with_symbols` takes another copy (the
// API's persistent checker, `ls_program::new_api_checker`).
// PORT: the checker reads its program through `program` and
// `compiler_options`, but the `program.rs` functions it calls read the
// current program (`prog()`). A thread that holds checkers of several
// programs makes the checker's program current while it uses the checker
// (`core::enter_program`).
// PORT: Go `make(map...)` initializations are the `Default` values in the
// struct literal; Go nil fields not set here keep their nil value.
