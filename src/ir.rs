// Lust IR: a small control-flow graph per function, in the style of Rust's MIR but flatter.
// The borrow checker and the C backend both read it.
use crate::ast::{BinOp, FmtSpec, UnOp};
use crate::check::Builtin;
use crate::def::LocalDefId;
use crate::diag::Span;
use crate::driver::graph::Reads;
use crate::ty::{AdtId, IntTy, Ty, Tys};

pub mod dropflags;

pub type Local = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Bb(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Proj {
    Deref,
    Field(u32),
    /// View an enum place as one variant, so `Field` can address that variant's fields.
    Downcast(u32),
    /// Bounds-checked Vec element. The index is in a local of type usize.
    /// Its panic location is the span of the local's last plain
    /// assignment (lowering's `mark_index_loc`; cgen's `index_locs`),
    /// not the span of the statement that uses the place.
    Index(Local),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub local: Local,
    pub proj: Vec<Proj>,
}

impl Place {
    pub fn local(l: Local) -> Place {
        Place { local: l, proj: Vec::new() }
    }
    pub fn project(&self, p: Proj) -> Place {
        let mut proj = self.proj.clone();
        proj.push(p);
        Place { local: self.local, proj }
    }
    pub fn deref(&self) -> Place {
        self.project(Proj::Deref)
    }
}

#[derive(Clone, Debug)]
pub enum Const {
    Int(u128, IntTy),
    /// A float literal as bits (`f32` bits zero-extended).
    Float(u64, crate::ty::FloatTy),
    Bool(bool),
    Char(char),
    /// Index into the AST string table.
    Str(u32),
    /// A synthesized `&str` value that is in no AST string table
    /// (a `type_name::<T>()` answer), as one shared buffer.
    StrBytes(crate::ast::ByteBlob),
    /// A byte array value (`[u8; N]`) as one shared buffer, never
    /// one operand per byte.
    Bytes(crate::ast::ByteBlob),
    Unit,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Copy(Place),
    Move(Place),
    Const(Const),
}

/// One piece of a format string. `Arg` holds a reference to the value and the value's type.
#[derive(Clone, Debug)]
pub enum FmtPart {
    Lit(String),
    /// `width` and `prec` are `&usize` operands for a width or precision
    /// taken from an argument (`spec.width_arg`, `spec.precision_arg`).
    Arg { value: Operand, ty: Ty, debug: bool, spec: FmtSpec, width: Option<Operand>, prec: Option<Operand> },
}

#[derive(Clone, Debug)]
pub enum Rvalue {
    Use(Operand),
    Ref(bool, Place),
    /// Integer arithmetic (checked in dev builds), bit operations, and scalar comparisons.
    Bin(BinOp, Operand, Operand),
    Un(UnOp, Operand),
    Cast(Operand, Ty),
    Aggregate(AdtId, u32, Vec<Operand>),
    /// `Some(x)` or `None`, of the given Option type.
    Opt(Ty, Option<Operand>),
    /// `Ok(x)` (true) or `Err(x)` (false), of the given Result type.
    Res(Ty, bool, Operand),
    VecLit(Ty, Vec<Operand>),
    VecRepeat(Ty, Operand, Operand),
    /// `a == b` for any PartialEq type. Both operands are references to `ty`.
    Eq {
        a: Operand,
        b: Operand,
        ty: Ty,
        ne: bool,
    },
    /// `a < b` (and `<=`, `>`, `>=`) for a type that orders by
    /// structure (`check::has_ord`). Both operands are references to
    /// `ty`; codegen compares through the `cmp` glue.
    Cmp {
        op: BinOp,
        a: Operand,
        b: Operand,
        ty: Ty,
    },
    /// String comparison. Both operands are `&str`.
    StrCmp(BinOp, Operand, Operand),
    /// `String + &str`.
    StringAdd(Operand, Operand),
    Format(Vec<FmtPart>),
    Discriminant(Place),
    /// A tuple value of the given type from its elements, in order (W-CORELANG, wave A).
    TupleAgg(Ty, Vec<Operand>),
    /// An array value of the given type from its elements, in order (W-CORELANG, wave A).
    /// Array repeat of a Copy element lowers to `ArrayRepeat` instead.
    ArrayAgg(Ty, Vec<Operand>),
    /// `[value; len]` of the given array type for a Copy element with len
    /// above 1 (W-CORELANG, wave A).
    ArrayRepeat(Ty, Operand, u64),
    /// A fat slice value over the whole of an array, Vec, or slice place
    /// (W-CORELANG, wave A). The result type is a slice reference type.
    SliceOf(Place),
    /// A fat slice value over a range of an array, Vec, or slice place.
    /// `hi` is the end operand when written; `inclusive` marks `..=` so the
    /// runtime helper can reject an overflowing end (W-CORELANG, wave A).
    Subslice {
        base: Place,
        lo: Operand,
        hi: Option<Operand>,
        inclusive: bool,
        /// True when the slice place is used mutably (`&mut a[..]`, a write).
        mutable: bool,
    },
    /// The element count of an array, slice, or Vec place (W-CORELANG, wave A).
    LenOf(Place),
    /// A copy of a static item's value, by index into `Program::statics`.
    StaticLoad(u32),
    /// The address of a static item, by index into `Program::statics`.
    StaticRef(u32),
    /// A function item used as a value, of the given result type: the
    /// zero-sized `FnDef`, or `FnPtr` when the use coerces it (codegen
    /// then yields the slot trampoline address). Proposed by W-CGEN
    /// for W-CORELANG's fn-pointer lowering.
    FnValue(LocalDefId, Ty),
    /// A constructor item (its `FnDef` type, `.0`: a tuple struct or
    /// variant, `Some`, `Ok`, `Err`) or an integer type's primitive
    /// builtin method item (`u32::isqrt`) coerced to fn pointer type
    /// `.1`. Codegen writes a small static C fn that builds the value
    /// or calls the builtin on its args (one per body and item), and
    /// the value is its address.
    ItemShim(Ty, Ty),
    /// A dyn reference from a thin one (ARCH 2.8): `ptr` is `&T` or
    /// `&mut T`, `src` is `T`, and `dyn_ty` the target dyn type. The
    /// result pairs the data pointer with the vtable of (`src`, the dyn
    /// principal), which mono built and codegen emits. `ptr` may also
    /// be a moved `Box<T>` or `Rc<T>`: the result is then the owning
    /// `Box<dyn Tr>` or `Rc<dyn Tr>`, whose data pointer points at the
    /// value (inside the Rc allocation, after its two counts).
    MakeDyn { ptr: Operand, src: Ty, dyn_ty: Ty },
    /// Trait upcasting: the same data pointer, with the vtable reached
    /// by loading each super slot word index in `path` in turn.
    DynUpcast { ptr: Operand, path: Vec<u32> },
    /// One header word of a dyn reference's vtable as a `usize`: the
    /// size or the align of the value behind it (`size_of_val` and
    /// `align_of_val` on a dyn value; word indexes in `check::object`).
    DynMeta { ptr: Operand, word: u32 },
    /// The data pointer of a dyn pointer (`&dyn`, `&mut dyn`,
    /// `Box<dyn>`, `Rc<dyn>`) as the thin pointer type `ty` (`&T`,
    /// `&mut T`, `Box<T>`, `Rc<T>`): a downcast after its type id test
    /// (lower/any.rs). The operand is moved for an owning pointer.
    DynData { ptr: Operand, ty: Ty },
    /// `dst.write_fmt(format_args!(..))` (W-STD-B): formats the parts
    /// into `dst` (a `&mut Formatter` or `&mut String`) with a fresh
    /// Formatter over its sink, stopping at the first error; the value
    /// is the `fmt::Result`. With `stream` set (`Some(true)` for
    /// stderr, `Some(false)` for stdout) the destination is one of
    /// std's stream types instead: each part goes to the locked stream
    /// as it is formatted (as `print!` does), a write error is the
    /// `io::Result`'s `Err`, and a fmt impl error panics as
    /// `io::Write::write_fmt` does in std.
    WriteFmt(Operand, Vec<FmtPart>, Option<bool>),
    /// A `format_args!` value (R9): an `fmt::Arguments` that formats
    /// `parts` each time it is formatted, as core's does. `env` is a
    /// `&` of the arguments temporary (a tuple of the argument
    /// references, whose scope is the one rustc gives its args
    /// array); each argument operand of `parts` reads a field of that
    /// temporary. None when no argument is left after rustc's literal
    /// inlining: the value is then `'static` text.
    FormatArgs(Option<Operand>, Vec<FmtPart>),
}

#[derive(Clone, Debug)]
pub enum StmtKind {
    Assign(Place, Rvalue),
    /// Drops the local if its drop flag says it still owns a value.
    Drop(Local),
    StorageDead(Local),
    /// A use that only the borrow checker sees. Keeps a two-phase reservation alive.
    FakeRead(Local),
    Print {
        stderr: bool,
        newline: bool,
        parts: Vec<FmtPart>,
    },
}

#[derive(Clone, Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

/// What a `Call` terminator calls: a user fn or method by local def, or a
/// builtin operation. Frozen with the `Call` terminator (docs/ARCH.md
/// section 9.2): new target forms go at the end only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallTarget {
    Fn(LocalDefId),
    Builtin(Builtin),
    /// A registry intrinsic by id (`ty::lang::INTRINSICS`, S5): one target
    /// for the whole family; codegen emits the row's C symbol with the
    /// row's signature.
    Intrinsic(u32),
    /// A call through the function pointer value held in this local.
    /// The local's type is the `FnPtr` signature; `args` are the call
    /// arguments only. Proposed by W-CGEN for W-CORELANG.
    Indirect(Local),
    /// A call through a vtable (ARCH 2.8): `args[0]` is the dyn
    /// receiver (`&dyn Tr` or `&mut dyn Tr`, a fat pointer), and the
    /// payload is the method's word index in that receiver's vtable.
    /// The callee gets the data pointer, then `args[1..]`.
    Dyn(u32),
}

#[derive(Clone, Debug)]
pub enum Terminator {
    Goto(Bb),
    If(Operand, Bb, Bb),
    /// Jumps on an enum discriminant.
    Switch(Operand, Vec<(u32, Bb)>, Bb),
    /// A call: writes its result to `dest`, then continues at `next`.
    /// `unwind` is the cleanup successor for a panic in the callee
    /// (docs/ARCH.md section 7.2); it is `None` everywhere until W-PANIC
    /// builds cleanup edges. Calls are terminators, never rvalues, so
    /// every call already has its unwind slot.
    Call {
        target: CallTarget,
        args: Vec<Operand>,
        dest: Place,
        next: Bb,
        unwind: Option<Bb>,
    },
    Return,
    Unreachable,
    /// The otherwise edge of an exhaustive `match`, after every arm's test failed: no
    /// value reaches it in a correct program, and rustc emits `unreachable` there. Every
    /// pass treats it as `Unreachable`; release C without debug assertions or overflow
    /// checks lets the C compiler assume it is never reached (`__builtin_unreachable`,
    /// LUST_RELEASE_UNREACHABLE=abort keeps the abort), every other build aborts.
    MatchExhausted,
    Panic(Vec<FmtPart>),
}

#[derive(Clone, Debug)]
pub struct BlockData {
    pub stmts: Vec<Stmt>,
    pub term: Terminator,
    pub term_span: Span,
}

#[derive(Clone, Debug)]
pub struct LocalDecl {
    pub ty: Ty,
    pub mutable: bool,
    /// User-visible name, for error messages. Temporaries have none.
    pub name: Option<String>,
}

/// Local 0 is the return value. Locals `1..=arg_count` are the parameters.
pub struct Body<'g> {
    pub fn_id: LocalDefId,
    pub tys: Tys<'g>,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BlockData>,
    pub arg_count: usize,
    #[allow(dead_code)] // Read by invalidation when it lands (W-DRIVER).
    /// The cross-item reads recorded while this body was checked
    /// (docs/ARCH.md section 8.2). Carried with the body result so the
    /// driver can store it with the checked body; record-only today.
    pub reads: Reads,
    /// Errors lowering found (a closure body place that no capture
    /// covers). Borrow checking reports them first; such a body is
    /// never generated.
    pub lower_errs: Vec<crate::diag::Diag>,
    /// `let` annotations with explicit lifetimes (W-BORROW): each local
    /// and the regioned type its annotation gives it. Borrowck relates
    /// the local's regions to it; a cached body has none (it is never
    /// borrow-checked again).
    pub annotations: Vec<(Local, Ty)>,
    /// Promoted temporaries (W-BORROW): the storage of a `&[]`, `&mut
    /// []`, `&|..| ..` with no captures, or a constant (`&5`, `&CONST`),
    /// which rustc promotes to `'static`, and the backing storage of a
    /// const reference value materialized in a body. Such a local never
    /// dies, its borrows make no loan, and cgen gives it C `static`
    /// storage, so a reference to it may outlive the function.
    pub promoted: Vec<Local>,
    /// The index temporaries of `a[i]` expressions. Borrowck never treats
    /// them as constant indexes (rustc's `Index`, not `ConstantIndex`):
    /// `arr[1]` reads the whole array for move checks. Only borrowck reads
    /// it; a cached body has none.
    pub index_exprs: Vec<Local>,
    /// Pattern-created index temporaries that hold `len - k` for a
    /// slice pattern's suffix element (rustc's `ConstantIndex`
    /// with `from_end`), as (local, k, minimum length). Borrowck treats two
    /// different back offsets of one slice as disjoint elements,
    /// and a back offset inside a pattern subslice's excluded
    /// suffix as disjoint from that subslice. Only borrowck
    /// reads it; a cached body has none.
    pub pat_back_index: Vec<(Local, u64, u64)>,
    /// Pattern-created index temporaries for a slice pattern's
    /// prefix elements, as (local, index, minimum length): the
    /// pattern's length test bounds the slice length below, so
    /// a prefix index and a suffix back offset of one pattern
    /// name disjoint elements when they meet the bound. Only
    /// borrowck reads it; a cached body has none.
    pub pat_front_index: Vec<(Local, u64, u64)>,
    /// The destination locals of pattern-created subslices (a
    /// slice pattern's middle). Borrowck gives such a loan the
    /// subslice's index range, so the pattern's own prefix and
    /// suffix element loans beside it are disjoint from it.
    /// Only borrowck reads it; a cached body has none.
    pub pat_subslice: Vec<Local>,
    /// The temporaries that hold a `&mut dyn Tr` coerced to a `&mut dyn
    /// Tr` at a rustc coercion site (check's `Coerce::Unsize`, lowering's
    /// `unsize`). Only there may borrowck shorten the object lifetime
    /// (`dyn Tr + 'a` to `dyn Tr + 'b`); elsewhere `&mut` keeps it
    /// invariant. Only borrowck reads it; a cached body has none.
    pub dyn_coerce: Vec<Local>,
    /// Edges only borrowck sees, as rustc's `FalseEdge`: from the point
    /// where a match arm's pattern has matched (before its bindings) to
    /// the next arm's start, so arms after an irrefutable one are still
    /// checked. Codegen ignores them; a cached body has none.
    pub false_edges: Vec<(Bb, Bb)>,
    /// Locals that may hold an `impl Trait` value revealed to its hidden
    /// type (check/opaque.rs): the value captures every region of its
    /// defining call. rustc's drop check assumes an opaque type has a
    /// destructor that uses every region it captures, so borrowck keeps
    /// all their regions live to the drop, which lowering emits even for
    /// a hidden type without drop glue (a no-op in codegen).
    pub opaque_locals: Vec<Local>,
    /// The `opaque_locals` whose drop runs no opaque destructor: every
    /// opaque they hold is `Copy` by its bounds, and their type has no
    /// other drop glue (`opaque::Hold::Copy`).
    pub opaque_nodrop: Vec<Local>,
    /// The cleanup entry of each statement that can unwind, as
    /// (block, statement index, cleanup block), in panic=unwind
    /// builds only (empty under panic=abort). An index equal to the
    /// block's statement count names its `Panic` terminator. A `Call`
    /// keeps its own `unwind` successor. Codegen checks for a new
    /// exception after each such statement (docs/ARCH.md section 7.2).
    pub stmt_unwind: Vec<(Bb, u32, Bb)>,
}

impl Body<'_> {
    pub fn block(&self, b: Bb) -> &BlockData {
        &self.blocks[b.0 as usize]
    }
}

/// Per local, the one byte string blob a temporary only ever holds:
/// an unnamed, non-parameter local whose every whole-local write is
/// that same `Const::Bytes`, which nothing writes through a
/// projection, a call result, or a mutable borrow. Borrowck treats
/// such a local's storage as never ending (a byte string literal is
/// `&'static [u8; N]` in Rust), and codegen gives it static storage,
/// so a borrow of it may escape the function (`-> &'static [u8]`).
pub fn byte_blob_locals(body: &Body) -> Vec<Option<crate::ast::ByteBlob>> {
    let n = body.locals.len();
    let mut blob: Vec<Option<crate::ast::ByteBlob>> = vec![None; n];
    let mut bad = vec![false; n];
    for (i, l) in body.locals.iter().enumerate() {
        if i <= body.arg_count || l.name.is_some() {
            bad[i] = true;
        }
    }
    let mut scan_rv = |rv: &Rvalue, bad: &mut Vec<bool>| match rv {
        Rvalue::Ref(true, p) => bad[p.local as usize] = true,
        Rvalue::Subslice { base, mutable: true, .. } => bad[base.local as usize] = true,
        _ => {}
    };
    for b in &body.blocks {
        for s in &b.stmts {
            if let StmtKind::Assign(dest, rv) = &s.kind {
                scan_rv(rv, &mut bad);
                let i = dest.local as usize;
                if dest.proj.is_empty()
                    && let Rvalue::Use(Operand::Const(Const::Bytes(bytes))) = rv
                {
                    match &blob[i] {
                        None => blob[i] = Some(bytes.clone()),
                        Some(prev) if prev.as_slice() == bytes.as_slice() => {}
                        Some(_) => bad[i] = true,
                    }
                } else {
                    bad[i] = true;
                }
            }
        }
        if let Terminator::Call { dest, .. } = &b.term {
            bad[dest.local as usize] = true;
        }
    }
    for i in 0..n {
        if bad[i] {
            blob[i] = None;
        }
    }
    blob
}

/// Collects the place of `o` when it is a move.
pub fn operand_moves<'r>(o: &'r Operand, out: &mut Vec<&'r Place>) {
    if let Operand::Move(pl) = o {
        out.push(pl);
    }
}

pub fn parts_moves<'r>(parts: &'r [FmtPart], out: &mut Vec<&'r Place>) {
    for part in parts {
        if let FmtPart::Arg { value, .. } = part {
            operand_moves(value, out);
        }
    }
}

/// Calls `op` for every operand one rvalue reads and `pl` for
/// every place it reads without an operand (a borrow, a
/// discriminant, a slice or length base). The one exhaustive
/// rvalue scan: cgen's move and mentions scans and the drop flag pass
/// (`dropflags`) all go through it, so a new `Rvalue` variant updates each.
pub fn visit_rvalue_reads<'r>(rv: &'r Rvalue, op: &mut impl FnMut(&'r Operand), pl: &mut impl FnMut(&'r Place)) {
    match rv {
        Rvalue::Use(o)
        | Rvalue::Un(_, o)
        | Rvalue::Cast(o, _)
        | Rvalue::ArrayRepeat(_, o, _)
        | Rvalue::Res(_, _, o)
        | Rvalue::MakeDyn { ptr: o, .. }
        | Rvalue::DynUpcast { ptr: o, .. }
        | Rvalue::DynMeta { ptr: o, .. }
        | Rvalue::DynData { ptr: o, .. } => op(o),
        Rvalue::Bin(_, a, b)
        | Rvalue::StrCmp(_, a, b)
        | Rvalue::StringAdd(a, b)
        | Rvalue::VecRepeat(_, a, b)
        | Rvalue::Eq { a, b, .. }
        | Rvalue::Cmp { a, b, .. } => {
            op(a);
            op(b);
        }
        Rvalue::Aggregate(_, _, ops) | Rvalue::VecLit(_, ops) | Rvalue::TupleAgg(_, ops) | Rvalue::ArrayAgg(_, ops) => {
            ops.iter().for_each(|o| op(o))
        }
        Rvalue::Opt(_, o) => {
            if let Some(o) = o {
                op(o);
            }
        }
        Rvalue::Format(parts) | Rvalue::FormatArgs(None, parts) => parts.iter().for_each(|p| {
            if let FmtPart::Arg { value, .. } = p {
                op(value);
            }
        }),
        Rvalue::WriteFmt(dst, parts, _) | Rvalue::FormatArgs(Some(dst), parts) => {
            op(dst);
            parts.iter().for_each(|p| {
                if let FmtPart::Arg { value, .. } = p {
                    op(value);
                }
            })
        }
        Rvalue::Ref(_, p) | Rvalue::Discriminant(p) | Rvalue::SliceOf(p) | Rvalue::LenOf(p) => pl(p),
        Rvalue::Subslice { base, lo, hi, .. } => {
            pl(base);
            op(lo);
            if let Some(h) = hi {
                op(h);
            }
        }
        Rvalue::StaticLoad(_) | Rvalue::StaticRef(_) | Rvalue::FnValue(..) | Rvalue::ItemShim(..) => {}
    }
}

/// Collects the moved places among the operands of one rvalue.
pub fn rvalue_moves<'r>(rv: &'r Rvalue, out: &mut Vec<&'r Place>) {
    visit_rvalue_reads(rv, &mut |o| operand_moves(o, out), &mut |_| {});
}

/// The successors of a block through its terminator (a call's unwind successor too).
pub fn succ(t: &Terminator) -> Vec<Bb> {
    match t {
        Terminator::Goto(b) => vec![*b],
        Terminator::If(_, a, b) => vec![*a, *b],
        Terminator::Switch(_, cases, other) => cases.iter().map(|c| c.1).chain(std::iter::once(*other)).collect(),
        Terminator::Call { next, unwind, .. } => std::iter::once(*next).chain(*unwind).collect(),
        Terminator::Return | Terminator::Unreachable | Terminator::MatchExhausted | Terminator::Panic(_) => Vec::new(),
    }
}
