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

use crate::jsnum::{Number, PseudoBigInt};
use crate::prelude::*;
use std::sync::LazyLock;

// Go: checker/checker.go:52 TypeSystemEntity
// PORT: Go `TypeSystemEntity any`. The Go callers of `pushTypeResolution`
// pass a declaration node, a symbol, a type or a signature, so the Rust type
// is an enum over those four handles. Go compares targets with `==` on the
// interface value, which is handle equality here.





// Go: checker/checker.go:69 TypeResolution

// ContextualInfo

// Go: checker/checker.go:77 ContextualInfo

// InferenceContextInfo

// Go: checker/checker.go:85 InferenceContextInfo

// EnumLiteralKey

// Go: checker/checker.go:103 EnumLiteralKey
// PORT: Go `value any` holds a `LiteralValue`. `LiteralValue` is not `Hash`
// (it holds `f64`), so the key stores `LiteralValueKey`, which compares like
// Go interface `==` (numbers as floats, -0 == +0). Build it with
// `LiteralValueKey::from(&value)`.

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

// CachedTypeKey

// Go: checker/checker.go:146 CachedTypeKey

// NarrowedTypeKey

// Go: checker/checker.go:153 NarrowedTypeKey

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

// AssignmentReducedKey

// Go: checker/checker.go:193 AssignmentReducedKey

// DiscriminatedContextualTypeKey

// Go: checker/checker.go:200 DiscriminatedContextualTypeKey
// PORT: Go `ast.NodeId` is the node handle here (a `Node` is unique per
// node, like a Go node id).

// InstantiationExpressionKey

// Go: checker/checker.go:207 InstantiationExpressionKey
// PORT: Go `ast.NodeId` is the node handle here.

// SubstitutionTypeKey

// Go: checker/checker.go:214 SubstitutionTypeKey

// ReverseMappedTypeKey

// Go: checker/checker.go:221 ReverseMappedTypeKey

// IterationTypesKey

// Go: checker/checker.go:229 IterationTypesKey

// PropertiesTypesKey

// Go: checker/checker.go:236 PropertiesTypesKey

// NonExistentPropertyKey

// Go: checker/checker.go:244 NonExistentPropertyKey

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



// PORT: `Default` exists only for the dummy entry at index 0 of
// `Checker::inference_contexts`. Go never has a nil `compareTypes`, so the
// placeholder comparer panics like a call of a nil Go func.

// Go: checker/checker.go:289 InferenceInfo
// PORT: Go `*InferenceInfo` is an index into `InferenceContext::inferences`.
// PERF (infermem1): 32 bytes (asserted below), not 72. The two candidate
// lists are in one box that the first candidate makes (typebox: about 6M
// infos, under a third of them with candidates). `candidates()` and
// `contra_candidates()` give Go's nil lists while it is absent.

// 32-bit targets (wasm32) have smaller pointers and do not check this.



// Go: checker/checker.go:321 IntraExpressionInferenceSite

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

// Go: checker/checker.go:521 IterationTypesResolver
// PORT: Go func fields are `Rc<dyn Fn(&mut Checker, ...)>`. Go
// `*diagnostics.Message` fields are always set by
// `initializeIterationResolvers`, so they are plain `&'static Message`.

// Go: checker/checker.go:537 WideningContext
// PORT: Go `*WideningContext` links become `Rc<RefCell<WideningContext>>`;
// a nil parent is `None`.

// Go: checker/checker.go:546 VarianceStackEntry

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
