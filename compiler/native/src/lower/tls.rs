// `LocalKey` method calls (check/tls.rs) in lowering and after borrowck.
//
// Lowering gives each key method call two more operands: the static's
// init fn as an `fn() -> T` pointer value, and whether the key is the
// `const` form (`Lower::tls_operands`). Both live in the IR, so a cached
// body or C unit can never pair a key with a stale form. Borrowck sees
// the call as one builtin call: the key borrow, the closure moved in, the
// result out. Codegen calls `expand` first, and each call becomes plain IR
// over the runtime accesses (`TlsAccess`, `TlsTryAccess`, `TlsInitWith`,
// emitted by cgen.rs): the value's `&T` (Rust's AccessError panic once it
// is destroyed, or a switch on `Option<&T>` for `try_with`) and the
// closure call or Cell and RefCell builtin.
//
// Every builder below starts in `self.cur` and leaves `self.cur` at the
// block that follows the value it wrote.
use super::Lower;
use crate::check::Builtin;
use crate::check::closure::ClosureKind;
use crate::check::collect::{Program, needs_drop};
use crate::diag::Span;
use crate::ir::*;
use crate::ty::*;

type R = Result<(), String>;

/// Rust 1.99's panic text for a key used during or after its destructor.
pub const ACCESS_PANIC: &str = "cannot access a Thread Local Storage value during or after destruction: AccessError";

/// True for the `LocalKey` method builtins: lowering adds the init fn
/// operand, and `expand` rewrites them.
pub fn is_key_method(b: Builtin) -> bool {
    use Builtin as B;
    matches!(
        b,
        B::TlsWith
            | B::TlsTryWith
            | B::TlsWithBorrow
            | B::TlsWithBorrowMut
            | B::TlsSet
            | B::TlsGet
            | B::TlsTake
            | B::TlsReplace
            | B::TlsTryGet
            | B::TlsCloneRc
    )
}

/// The key methods that take no operand of their own, even when written
/// with a std fn path argument (`KEY.with(Cell::get)` is `TlsGet`):
/// lowering skips their arguments.
pub fn takes_no_args(b: Builtin) -> bool {
    matches!(b, Builtin::TlsGet | Builtin::TlsTryGet | Builtin::TlsCloneRc | Builtin::TlsTake)
}

impl Lower<'_> {
    /// The extra operands of a key method call: the init fn of the
    /// `thread_local!` static its receiver names (the checker allows no
    /// other receiver), as an `fn() -> T` pointer value, then the key's
    /// `const` form flag.
    pub(super) fn tls_operands(&mut self, recv: crate::ast::ExprId, span: Span) -> [Operand; 2] {
        let crate::check::Res::Static(idx) = self.ck.res(recv) else { unreachable!("check/tls.rs: a key method names its static") };
        let st = &self.p.statics[idx as usize];
        let (key, ty) = (st.tls.expect("check/tls.rs: a key method names a thread_local! static"), st.ty);
        let t = self.body.tys.local_key_elem(ty).expect("a thread_local! static is a LocalKey");
        let inputs = self.body.tys.intern_tys(&[]);
        let sig = self.body.tys.intern_sig(Binder::plain(FnSig::plain(inputs, t)));
        let fp = self.body.tys.intern(TyKind::FnPtr(sig));
        let l = self.temp(fp);
        self.assign(Place::local(l), Rvalue::FnValue(key.init, fp), span);
        [Operand::Copy(Place::local(l)), Operand::Const(Const::Bool(key.eager))]
    }
}

/// The body with every key method call expanded, or None when it has
/// none (the common case: no copy is made).
pub fn expand<'g>(p: &Program, body: &Body<'g>) -> Result<Option<Body<'g>>, String> {
    let has = body.blocks.iter().any(|b| matches!(b.term, Terminator::Call { target: CallTarget::Builtin(x), .. } if is_key_method(x)));
    if !has {
        return Ok(None);
    }
    let copy = Body {
        fn_id: body.fn_id,
        tys: body.tys.clone(),
        locals: body.locals.clone(),
        blocks: body.blocks.clone(),
        arg_count: body.arg_count,
        reads: Default::default(),
        lower_errs: Vec::new(),
        annotations: Vec::new(),
        // cgen gives these static storage: a reference to one may escape.
        promoted: body.promoted.clone(),
        index_exprs: Vec::new(),
        pat_back_index: Vec::new(),
        pat_front_index: Vec::new(),
        pat_subslice: Vec::new(),
        dyn_coerce: Vec::new(),
        false_edges: Vec::new(),
        opaque_locals: Vec::new(),
        opaque_nodrop: Vec::new(),
        stmt_unwind: body.stmt_unwind.clone(),
    };
    let mut ex = Ex { p, body: copy, cur: Bb(0), span: Span::default(), access_site: None, unw: None };
    for b in 0..ex.body.blocks.len() {
        let Terminator::Call { target: CallTarget::Builtin(x), .. } = ex.body.blocks[b].term else { continue };
        if !is_key_method(x) {
            continue;
        }
        let term = std::mem::replace(&mut ex.body.blocks[b].term, Terminator::Unreachable);
        let Terminator::Call { args, dest, next, unwind, .. } = term else { unreachable!() };
        ex.cur = Bb(b as u32);
        ex.span = ex.body.blocks[b].term_span;
        ex.unw = unwind;
        ex.method(x, args, dest)?;
        ex.goto(next);
    }
    Ok(Some(ex.body))
}

struct Ex<'b, 'g> {
    p: &'b Program,
    body: Body<'g>,
    cur: Bb,
    span: Span,
    /// The local.rs line and column the next `access` reports a
    /// destroyed value at, when not `with`'s (see `set`).
    access_site: Option<(u32, u32)>,
    /// The expanded call's cleanup successor (as in lower/iter.rs).
    unw: Option<Bb>,
}

impl Ex<'_, '_> {
    // ---------- IR building ----------

    fn local(&mut self, ty: Ty) -> Local {
        self.body.locals.push(LocalDecl { ty, mutable: true, name: None });
        self.body.locals.len() as Local - 1
    }
    fn block(&mut self) -> Bb {
        self.body.blocks.push(BlockData { stmts: Vec::new(), term: Terminator::Unreachable, term_span: self.span });
        Bb(self.body.blocks.len() as u32 - 1)
    }
    fn assign(&mut self, pl: Place, rv: Rvalue) {
        self.push(StmtKind::Assign(pl, rv));
    }
    fn drop_local(&mut self, l: Local) {
        self.push(StmtKind::Drop(l));
    }
    /// Appends a statement; under panic=unwind it continues at the
    /// expanded call's cleanup successor when it panics.
    fn push(&mut self, kind: StmtKind) {
        let span = self.span;
        let stmts = &mut self.body.blocks[self.cur.0 as usize].stmts;
        stmts.push(Stmt { kind, span });
        if let Some(u) = self.unw.filter(|_| self.p.unwind) {
            let n = stmts.len() as u32 - 1;
            self.body.stmt_unwind.push((self.cur, n, u));
        }
    }
    fn term(&mut self, t: Terminator) {
        let b = &mut self.body.blocks[self.cur.0 as usize];
        b.term = t;
        b.term_span = self.span;
    }
    fn goto(&mut self, b: Bb) {
        self.term(Terminator::Goto(b));
    }
    /// Runs `f` with its statements' span at line `line`, col `col` of
    /// std's thread/local.rs, where rustc 1.99 reports the panics of
    /// these key methods (W-PANIC; cgen's `std_loc` file 10).
    fn at_local_rs<T>(&mut self, line: u32, col: u32, f: impl FnOnce(&mut Self) -> T) -> T {
        let saved = std::mem::replace(&mut self.span, Span::std_site(10, line, col));
        let r = f(self);
        self.span = saved;
        r
    }
    /// Ends the current block with a call and continues after it.
    fn call(&mut self, target: CallTarget, args: Vec<Operand>, dest: Place) {
        let next = self.block();
        self.term(Terminator::Call { target, args, dest, next, unwind: self.unw });
        self.cur = next;
    }
    /// The type of an operand. Every operand of a key method call is a
    /// place (a closure, a captured closure's env field, a temporary).
    fn op_ty(&mut self, o: &Operand) -> Result<Ty, String> {
        match o {
            Operand::Copy(pl) | Operand::Move(pl) => self.place_ty(pl),
            Operand::Const(_) => Err("thread_local expansion: a constant operand".into()),
        }
    }

    /// The type of a place: its local's type through the projections
    /// lowering makes for operands and destinations (derefs, fields of
    /// tuples, closures, Adts, and enum variants).
    fn place_ty(&mut self, pl: &Place) -> Result<Ty, String> {
        let mut t = self.body.locals[pl.local as usize].ty;
        let mut variant: Option<u32> = None;
        for pr in &pl.proj {
            let tys = &mut self.body.tys;
            t = match *pr {
                Proj::Deref => match tys.kind(t) {
                    TyKind::Ref(_, _, i) => i,
                    _ => tys.smart_elem(t).ok_or("thread_local expansion: deref of a non-pointer place")?,
                },
                Proj::Downcast(v) => {
                    variant = Some(v);
                    continue;
                }
                Proj::Field(i) => {
                    let v = variant.take();
                    if let Some(e) = tys.option_elem(t) {
                        e
                    } else if let Some((a, b)) = tys.result_args(t) {
                        if v == Some(1) { b } else { a }
                    } else if let TyKind::Tuple(ts) = tys.kind(t) {
                        tys.tys_of(ts)[i as usize]
                    } else if let TyKind::Closure(_, a) = tys.kind(t) {
                        crate::check::collect::closure_fields(tys, a)[i as usize]
                    } else if let Some(a) = tys.local_adt(t) {
                        self.p.adt(a).variants[v.unwrap_or(0) as usize].fields[i as usize].ty
                    } else {
                        return Err("thread_local expansion: field of an unknown place type".into());
                    }
                }
                Proj::Index(_) => match tys.kind(t) {
                    TyKind::Slice(e) | TyKind::Array(e, _) => e,
                    _ => tys.vec_elem(t).ok_or("thread_local expansion: index of a non-sequence place")?,
                },
            };
        }
        Ok(t)
    }
    /// Moves an operand into a fresh local of its own.
    fn own(&mut self, o: Operand) -> Result<(Local, Ty), String> {
        let ty = self.op_ty(&o)?;
        Ok((self.own_as(o, ty), ty))
    }
    /// Moves an operand of known type `ty` (a constant too) into a fresh
    /// local of its own.
    fn own_as(&mut self, o: Operand, ty: Ty) -> Local {
        let l = self.local(ty);
        self.assign(Place::local(l), Rvalue::Use(o));
        l
    }

    // ---------- Key access ----------

    /// `&T` of the key's value into a fresh local: the runtime access
    /// (which runs the init fn on the thread's first access), with
    /// Rust's AccessError panic when the value is destroyed. With
    /// `on_none`, the destroyed case runs it instead of the panic and
    /// goes on at the returned join block.
    fn access(&mut self, key: &Operand, init: &Operand, t: Ty, on_none: Option<&mut dyn FnMut(&mut Self) -> R>) -> Result<(Local, Option<Bb>), String> {
        let rt = self.body.tys.mk_ref(false, t);
        let Some(on_none) = on_none else {
            let r = self.local(rt);
            // `with`'s `expect` in local.rs (`set` passes its own site).
            let (line, col) = self.access_site.take().unwrap_or((429, 25));
            self.at_local_rs(line, col, |this| this.call(CallTarget::Builtin(Builtin::TlsAccess), vec![key.clone(), init.clone()], Place::local(r)));
            return Ok((r, None));
        };
        let ot = self.body.tys.mk_option(rt);
        let o = self.local(ot);
        self.call(CallTarget::Builtin(Builtin::TlsTryAccess), vec![key.clone(), init.clone()], Place::local(o));
        let d = self.local(int(IntTy::U32));
        self.assign(Place::local(d), Rvalue::Discriminant(Place::local(o)));
        let (some, none) = (self.block(), self.block());
        self.term(Terminator::Switch(Operand::Copy(Place::local(d)), vec![(1, some)], none));
        self.cur = none;
        on_none(self)?;
        let join = self.block();
        self.goto(join);
        self.cur = some;
        let r = self.local(rt);
        self.assign(Place::local(r), Rvalue::Use(Operand::Copy(Place::local(o).project(Proj::Downcast(1)).project(Proj::Field(0)))));
        Ok((r, Some(join)))
    }

    /// `dest = f(args..)` through `FnOnce` for the callable in local
    /// `f`: a closure by its own kind (an `Fn` or `FnMut` closure goes by
    /// reference and drops after the call, as `call_once` does), a fn
    /// item, or a fn pointer.
    fn call_once(&mut self, f: Local, fty: Ty, args: Vec<Operand>, dest: Place) -> R {
        match self.body.tys.kind(fty) {
            TyKind::Closure(def, _) => {
                let entry = self.p.gx.closures.get(&def).ok_or("thread_local expansion: closure body not registered")?;
                let (body_fn, kind) = (entry.body_fn, entry.kind);
                let env = match kind {
                    ClosureKind::FnOnce => Operand::Move(Place::local(f)),
                    ClosureKind::Fn | ClosureKind::FnMut => {
                        let m = kind == ClosureKind::FnMut;
                        let rt = self.body.tys.mk_ref(m, fty);
                        let r = self.local(rt);
                        self.assign(Place::local(r), Rvalue::Ref(m, Place::local(f)));
                        Operand::Move(Place::local(r))
                    }
                };
                let mut ops = vec![env];
                ops.extend(args);
                self.call(CallTarget::Fn(body_fn), ops, dest);
                if kind != ClosureKind::FnOnce && needs_drop(self.p, &self.body.tys, fty) {
                    self.drop_local(f);
                }
                Ok(())
            }
            TyKind::FnPtr(_) => {
                self.call(CallTarget::Indirect(f), args, dest);
                Ok(())
            }
            TyKind::FnDef(def, fargs) => match self.p.local_def(def.hash) {
                Some(local) => {
                    self.call(CallTarget::Fn(local), args, dest);
                    Ok(())
                }
                // A std method item as the callable (`try_with(Cell::take)`):
                // its builtin, whose code is the FnDef's third arg
                // (check `prim_item_value`).
                None if def == crate::ty::prim_fn_def() => {
                    let code = match self.body.tys.args(fargs).get(2) {
                        Some(GenericArg::Const(c)) => match self.body.tys.const_kind(*c) {
                            ConstKind::Int(v) => v as u32,
                            _ => u32::MAX,
                        },
                        _ => u32::MAX,
                    };
                    match crate::check::prim_builtin_of_code(code) {
                        Some(bf) if bf != Builtin::PrimFromCast => {
                            self.call(CallTarget::Builtin(bf), args, dest);
                            Ok(())
                        }
                        _ => Err("thread_local expansion: this std fn item as the callable".into()),
                    }
                }
                None => Err("thread_local expansion: fn item outside this crate".into()),
            },
            _ => Err("thread_local expansion: the callable is not a closure or fn".into()),
        }
    }

    // ---------- Methods ----------

    /// One key method call: `args` are the key reference, the method's
    /// own arguments, then the init fn and the `const` form flag (see
    /// `Lower::tls_operands`).
    fn method(&mut self, b: Builtin, mut args: Vec<Operand>, dest: Place) -> R {
        use Builtin as B;
        let Some(Operand::Const(Const::Bool(eager))) = args.pop() else { return Err("thread_local expansion: missing key form".into()) };
        let init = args.pop().ok_or("thread_local expansion: missing init fn")?;
        let key = args.first().cloned().ok_or("thread_local expansion: missing key")?;
        let init_ty = self.op_ty(&init)?;
        let TyKind::FnPtr(sig) = self.body.tys.kind(init_ty) else { return Err("thread_local expansion: init is not a fn pointer".into()) };
        let t = self.body.tys.sig(sig).value.output;
        match b {
            B::TlsWith => {
                let (f, fty) = self.own(args.get(1).cloned().ok_or("thread_local expansion: missing closure")?)?;
                let (r, _) = self.access(&key, &init, t, None)?;
                self.call_once(f, fty, vec![Operand::Copy(Place::local(r))], dest)
            }
            B::TlsTryWith => {
                let dest_ty = self.place_ty(&dest)?;
                let (rt, _) = self.body.tys.result_args(dest_ty).ok_or("thread_local expansion: try_with result")?;
                let (f, fty) = self.own(args.get(1).cloned().ok_or("thread_local expansion: missing closure")?)?;
                let drop_f = needs_drop(self.p, &self.body.tys, fty);
                let err_dest = dest.clone();
                let mut on_none = |this: &mut Self| -> R {
                    // `try_with` returns before calling `f`, which drops.
                    if drop_f {
                        this.drop_local(f);
                    }
                    this.assign(err_dest.clone(), Rvalue::Res(dest_ty, false, Operand::Const(Const::Unit)));
                    Ok(())
                };
                let (r, join) = self.access(&key, &init, t, Some(&mut on_none))?;
                let out = self.local(rt);
                self.call_once(f, fty, vec![Operand::Copy(Place::local(r))], Place::local(out))?;
                self.assign(dest, Rvalue::Res(dest_ty, true, Operand::Move(Place::local(out))));
                let join = join.expect("try_with has a join block");
                self.goto(join);
                self.cur = join;
                Ok(())
            }
            B::TlsWithBorrow | B::TlsWithBorrowMut => {
                // `self.with(|cell| f(&cell.borrow()))`: the guard lives
                // until `f` returns.
                let m = b == B::TlsWithBorrowMut;
                let u = self.body.tys.refcell_elem(t).ok_or("thread_local expansion: with_borrow on a non-RefCell")?;
                let (f, fty) = self.own(args.get(1).cloned().ok_or("thread_local expansion: missing closure")?)?;
                let (r, _) = self.access(&key, &init, t, None)?;
                let g = self.guard(r, u, m, if m { (714, 38) } else { (681, 34) });
                let ut = self.body.tys.mk_ref(m, u);
                let ur = self.local(ut);
                self.assign(Place::local(ur), Rvalue::Ref(m, Place::local(g).deref()));
                self.call_once(f, fty, vec![Operand::Move(Place::local(ur))], dest)?;
                self.drop_local(g);
                Ok(())
            }
            B::TlsTryGet => {
                // `try_with(Cell::get)`: Ok(the copy), or Err once destroyed.
                let dest_ty = self.place_ty(&dest)?;
                let (u, _) = self.body.tys.result_args(dest_ty).ok_or("thread_local expansion: try_with result")?;
                let err_dest = dest.clone();
                let mut on_none = |this: &mut Self| -> R {
                    this.assign(err_dest.clone(), Rvalue::Res(dest_ty, false, Operand::Const(Const::Unit)));
                    Ok(())
                };
                let (r, join) = self.access(&key, &init, t, Some(&mut on_none))?;
                let out = self.local(u);
                self.call(CallTarget::Builtin(B::CellGet), vec![Operand::Copy(Place::local(r))], Place::local(out));
                self.assign(dest, Rvalue::Res(dest_ty, true, Operand::Move(Place::local(out))));
                let join = join.expect("try_with has a join block");
                self.goto(join);
                self.cur = join;
                Ok(())
            }
            B::TlsCloneRc => {
                // `with(Rc::clone)`: a new strong reference.
                let (r, _) = self.access(&key, &init, t, None)?;
                self.call(CallTarget::Builtin(B::Clone), vec![Operand::Copy(Place::local(r))], dest);
                Ok(())
            }
            B::TlsGet | B::TlsTake | B::TlsReplace => {
                // `self.with(|cell| cell.get())` and kin.
                let (r, _) = self.access(&key, &init, t, None)?;
                let refcell = self.body.tys.refcell_elem(t).is_some();
                let (op, ops) = match (b, refcell) {
                    (B::TlsGet, _) => (B::CellGet, vec![]),
                    (B::TlsTake, false) => (B::CellTake, vec![]),
                    (B::TlsTake, true) => (B::RefCellTake, vec![]),
                    (_, false) => (B::CellReplace, vec![args.get(1).cloned().ok_or("thread_local expansion: missing value")?]),
                    (_, true) => (B::RefCellReplace, vec![args.get(1).cloned().ok_or("thread_local expansion: missing value")?]),
                };
                let mut all = vec![Operand::Copy(Place::local(r))];
                all.extend(ops);
                // `RefCell::replace` reports its call in local.rs; `take`
                // goes through cell.rs (cgen's RefCellTake site).
                if op == B::RefCellReplace {
                    self.at_local_rs(820, 31, |this| this.call(CallTarget::Builtin(op), all, dest));
                } else {
                    self.call(CallTarget::Builtin(op), all, dest);
                }
                Ok(())
            }
            B::TlsSet => self.set(&key, &init, t, eager, args.get(1).cloned().ok_or("thread_local expansion: missing value")?, dest),
            _ => Err(format!("thread_local expansion: `{b:?}` is not a key method")),
        }
    }

    /// A `RefCell` guard over the cell at `&RefCell<U>` local `r`
    /// (`borrow` or `borrow_mut`, with their panics), in a fresh local.
    /// Its panic reports local.rs line `site` (rustc's borrow inside the
    /// key method).
    fn guard(&mut self, r: Local, u: Ty, m: bool, site: (u32, u32)) -> Local {
        let gt = self.body.tys.mk_builtin1(if m { |d| d.ref_mut_guard } else { |d| d.ref_guard }, u);
        let g = self.local(gt);
        let op = if m { Builtin::RefCellBorrowMut } else { Builtin::RefCellBorrow };
        self.at_local_rs(site.0, site.1, |this| this.call(CallTarget::Builtin(op), vec![Operand::Copy(Place::local(r))], Place::local(g)));
        g
    }

    /// `set(value)` on a `Cell` or `RefCell` key, as Rust's
    /// `initialize_with`: a lazy key not yet made on this thread takes
    /// the new cell as its value and never runs its initializer; else
    /// the value goes into the live cell (`Cell::set`, or
    /// `*cell.borrow_mut() = value`, which drops the old value while the
    /// cell is borrowed). A `const` key is made from the start.
    fn set(&mut self, key: &Operand, init: &Operand, t: Ty, eager: bool, value: Operand, dest: Place) -> R {
        use Builtin as B;
        let refcell = self.body.tys.refcell_elem(t);
        let u = refcell.or_else(|| self.body.tys.cell_elem(t)).ok_or("thread_local expansion: set on a non-cell key")?;
        let vl = self.own_as(value, u);
        let mut join = None;
        if !eager {
            let cl = self.local(t);
            let new = if refcell.is_some() { B::RefCellNew } else { B::CellNew };
            self.call(CallTarget::Builtin(new), vec![Operand::Move(Place::local(vl))], Place::local(cl));
            let ot = self.body.tys.mk_option(t);
            let back = self.local(ot);
            self.call(CallTarget::Builtin(B::TlsInitWith), vec![key.clone(), Operand::Move(Place::local(cl))], Place::local(back));
            let d = self.local(int(IntTy::U32));
            self.assign(Place::local(d), Rvalue::Discriminant(Place::local(back)));
            let (some, none) = (self.block(), self.block());
            self.term(Terminator::Switch(Operand::Copy(Place::local(d)), vec![(1, some)], none));
            join = Some(none);
            // The cell came back: the key was made already. Its value
            // goes into the live cell below.
            self.cur = some;
            let c2 = self.local(t);
            self.assign(Place::local(c2), Rvalue::Use(Operand::Move(Place::local(back).project(Proj::Downcast(1)).project(Proj::Field(0)))));
            let into = if refcell.is_some() { B::RefCellIntoInner } else { B::CellIntoInner };
            self.call(CallTarget::Builtin(into), vec![Operand::Move(Place::local(c2))], Place::local(vl));
        }
        // `initialize_with`'s own `expect` (local.rs 487:25).
        self.access_site = Some((487, 25));
        let (r, _) = self.access(key, init, t, None)?;
        if refcell.is_some() {
            let g = self.guard(r, u, true, (752, 23));
            self.assign(Place::local(g).deref(), Rvalue::Use(Operand::Move(Place::local(vl))));
            self.drop_local(g);
        } else {
            let unit = self.local(UNIT);
            self.call(CallTarget::Builtin(B::CellSet), vec![Operand::Copy(Place::local(r)), Operand::Move(Place::local(vl))], Place::local(unit));
        }
        if let Some(j) = join {
            self.goto(j);
            self.cur = j;
        }
        self.assign(dest, Rvalue::Use(Operand::Const(Const::Unit)));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    /// The runtime's thread-exit path (runtime/thread.c) by itself: each
    /// thread's destructors run newest first when it ends, a destructor
    /// sees a key destroyed before it, and the main thread's run at
    /// `lust_tls_exit`. A C harness stands in for codegen: keys with slots
    /// 1 and 2, `(u8, int)` cells. tests/pass/tls_threads.rs covers the
    /// same path through `thread::spawn`.
    #[test]
    fn thread_exit_runs_destructors_newest_first() {
        let harness = r#"
#include <stdint.h>
#include <stdio.h>
#include <pthread.h>
void *lust_tls_cell(uint64_t *key, uint64_t size, uint64_t align);
void lust_tls_register(void *cell, void (*dtor)(void *));
void lust_tls_exit(void);
uint64_t key_a = 1, key_b = 2;
typedef struct { uint8_t state; int v; } cell_t;
static void dtor(void *p) {
    cell_t *c = p;
    if (c->state != 1) return;
    c->state = 2;
    cell_t *b = lust_tls_cell(&key_b, sizeof(cell_t), _Alignof(cell_t));
    printf("drop %d b=%d\n", c->v, b->state);
}
static void touch(uint64_t *key, int v) {
    cell_t *c = lust_tls_cell(key, sizeof(cell_t), _Alignof(cell_t));
    if (c->state == 0) { c->v = v; c->state = 1; lust_tls_register(c, dtor); }
}
static void *worker(void *arg) {
    long id = (long)arg;
    touch(&key_a, (int)(id * 10 + 1));
    touch(&key_b, (int)(id * 10 + 2));
    touch(&key_a, 0);
    return 0;
}
int main(void) {
    for (long i = 1; i <= 2; i++) {
        pthread_t t;
        pthread_create(&t, 0, worker, (void *)i);
        pthread_join(t, 0);
    }
    touch(&key_a, 1);
    printf("main exit\n");
    lust_tls_exit();
    return 0;
}
"#;
        let dir = std::env::temp_dir().join(format!("lust-tls-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (rt, h) = (dir.join("rt.c"), dir.join("harness.c"));
        std::fs::write(&rt, crate::cgen::runtime_unit()).unwrap();
        std::fs::write(&h, harness).unwrap();
        let exe = dir.join("prog");
        let cc = std::process::Command::new("cc").args(["-O0", "-w", "-pthread", "-o"]).arg(&exe).arg(&h).arg(&rt).arg("-lm").output().unwrap();
        assert!(cc.status.success(), "cc failed: {}", String::from_utf8_lossy(&cc.stderr));
        let run = std::process::Command::new(&exe).output().unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(run.status.success(), "run failed: {}", String::from_utf8_lossy(&run.stderr));
        let want = "drop 12 b=2\ndrop 11 b=2\ndrop 22 b=2\ndrop 21 b=2\nmain exit\ndrop 1 b=0\n";
        assert_eq!(String::from_utf8(run.stdout).unwrap(), want);
    }
}
