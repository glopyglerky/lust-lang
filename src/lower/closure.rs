// Closure lowering (docs/ARCH.md 2.9, 4.5): the closure value, calls
// of closure values, and closure body fns.
//
// A closure value is a tuple of its captures (`TupleAgg` with the
// closure type). A body fn takes the env first: `&C` for `Fn`,
// `&mut C` for `FnMut`, `C` by value for `FnOnce`. An `FnOnce` body
// moves each capture into its own local at entry, so moves out of a
// capture are whole-local moves and the captures it leaves drop at
// body end, after the parameters, in capture order (rustc drops the
// by-value `self` last, field by field).
//
// Captured variables lower as "shadow" locals (the parent's locals,
// never assigned here). A pass after the body rewrites every place
// rooted at one onto the env: the capture whose path is a prefix of
// the place, plus the rest of the path. A place no capture covers is
// a mismatch between capture analysis and lowering: it becomes a
// lowering error (L0820), never silent code.
use super::*;
use crate::check::closure::{CaptureMode, ClosureInfo, ClosureKind};
use crate::diag::Diag;

/// Closure body state: shadow locals and the env place of each capture.
pub(super) struct Upvars {
    info: ClosureInfo,
    /// Shadow local to checker root local.
    shadow: Vec<(Local, u32)>,
    /// Per capture: the place holding the capture value (an env field,
    /// or the `FnOnce` capture local).
    field: Vec<Place>,
}

impl Upvars {
    /// True when `l` is the shadow local of a captured variable.
    pub(super) fn shadows(&self, l: Local) -> bool {
        self.shadow.iter().any(|&(s, _)| s == l)
    }
}

impl Upvars {
    fn root_of(&self, l: Local) -> Option<u32> {
        self.shadow.iter().find(|(s, _)| *s == l).map(|(_, r)| *r)
    }
}

/// Lowers a closure body fn (`fid` is registered in `p.gx.closure_fns`)
/// from its parent's checked body.
pub fn lower_closure<'p>(p: &'p Program, fid: LocalDefId, mut ck: Checked<'p>) -> Body<'p> {
    let f = p.fn_info(fid);
    let def = p.gx.closure_fns[&fid];
    let tys = lift_to_owned(std::mem::replace(&mut ck.tys, Tys::new(&p.tys)));
    let reads = std::mem::take(&mut ck.reads);
    let entry = BlockData { stmts: Vec::new(), term: Terminator::Unreachable, term_span: Span::default() };
    let body = Body {
        fn_id: fid,
        tys,
        locals: Vec::new(),
        blocks: vec![entry],
        arg_count: f.params.len(),
        reads,
        lower_errs: Vec::new(),
        annotations: Vec::new(),
        promoted: Vec::new(),
        index_exprs: Vec::new(),
        pat_back_index: Vec::new(),
        pat_front_index: Vec::new(),
        pat_subslice: Vec::new(),
        dyn_coerce: Vec::new(),
        false_edges: Vec::new(),
        opaque_locals: Vec::new(),
        opaque_nodrop: Vec::new(),
        stmt_unwind: Vec::new(),
    };
    let info = ck.closures.iter().find(|c| c.def == def).cloned();
    let poisoned = ck.poisoned;
    let mut lw = Lower {
        p,
        ck,
        body,
        cur: Bb(0),
        scopes: vec![Vec::new()],
        loops: Vec::new(),
        local_map: Vec::new(),
        mut_index: 0,
        cow_view: None,
        making_cow_view: None,
        drop_ty: Default::default(),
        upvars: None,
        guard_places: Vec::new(),
        or_tags: Vec::new(),
        or_forced: Vec::new(),
        guard_temps: Vec::new(),
        bind_reuse: Vec::new(),
        bind_depth: 0,
        prelowered: Vec::new(),
        spine_gen: 0,
        extended: Vec::new(),
        unwind_steps: Default::default(),
        unwind_end: None,
        unwind_last: None,
        in_cleanup: false,
        bind_of: Vec::new(),
        as_code: None,
    };
    let Some(info) = info.filter(|_| !poisoned) else {
        // The parent failed to check and reports why; keep the shape
        // (return slot and parameters) and nothing else.
        for &t in std::iter::once(&f.ret).chain(f.params.iter()) {
            lw.new_local(t, true, None);
        }
        return lw.body;
    };
    let Expr::Closure { params, body: body_expr, .. } = p.ast.expr(info.expr) else { unreachable!("closure info names a closure") };
    lw.new_local(info.output, true, None);
    lw.local_map = vec![u32::MAX; lw.ck.locals.len()];
    lw.guard_places = vec![None; lw.ck.locals.len()];
    let env_ty = match info.kind {
        ClosureKind::Fn => lw.body.tys.mk_ref(false, info.ty),
        ClosureKind::FnMut => lw.body.tys.mk_ref(true, info.ty),
        ClosureKind::FnOnce => info.ty,
    };
    let env = lw.new_local(env_ty, true, Some("{closure env}".to_string()));
    let param_locals: Vec<Local> = info.inputs.iter().map(|&t| lw.new_local(t, false, None)).collect();
    let span = p.ast.expr_span(info.expr);
    // Capture places. `FnOnce` moves every capture into a local first;
    // they go into the root scope in reverse so they drop in capture
    // order after everything else.
    let mut field: Vec<Place> = Vec::with_capacity(info.captures.len());
    let mut cap_locals: Vec<Local> = Vec::new();
    for (i, cap) in info.captures.iter().enumerate() {
        let fty = crate::check::closure::capture_field_ty(&mut lw.body.tys, cap);
        if info.kind == ClosureKind::FnOnce {
            let root = &lw.ck.locals[cap.root as usize];
            let (m, name) = (cap.mode == CaptureMode::Value && root.mutable, p.name(root.name).to_string());
            let l = lw.new_local(fty, m, Some(name));
            let op = lw.operand_from(Place::local(env).project(Proj::Field(i as u32)), fty);
            lw.assign(Place::local(l), Rvalue::Use(op), span);
            cap_locals.push(l);
            field.push(Place::local(l));
        } else {
            field.push(Place::local(env).deref().project(Proj::Field(i as u32)));
        }
    }
    for &l in cap_locals.iter().rev() {
        lw.scopes[0].push(l);
    }
    for &l in &param_locals {
        lw.scopes[0].push(l);
    }
    // Shadow locals for the captured roots.
    let mut shadow: Vec<(Local, u32)> = Vec::new();
    for cap in &info.captures {
        if shadow.iter().any(|(_, r)| *r == cap.root) {
            continue;
        }
        let root = &lw.ck.locals[cap.root as usize];
        let (t, m, name) = (root.ty, root.mutable, p.name(root.name).to_string());
        let s = lw.new_local(t, m, Some(name));
        lw.local_map[cap.root as usize] = s;
        shadow.push((s, cap.root));
    }
    lw.upvars = Some(Upvars { info: info.clone(), shadow, field });
    for (prm, &l) in params.iter().zip(&param_locals) {
        lw.bind_pat(prm.pat, Place::local(l));
    }
    lw.expr_into(*body_expr, Some(&Place::local(0)));
    lw.pop_scope(span);
    lw.terminate(Terminator::Return, span);
    lw.map_annotations();
    lw.rewrite_upvars();
    lw.mark_opaque_locals();
    lw.body
}

impl<'p> Lower<'p> {
    /// The closure value of one closure expression: a tuple of its
    /// captures, borrowed or moved per capture mode.
    pub(super) fn closure_value(&mut self, e: ExprId, dest: Option<&Place>, span: Span) {
        let ty = self.ty(e);
        let Some(info) = self.ck.closures.iter().find(|c| c.expr == e).cloned() else { return };
        // A closure checked as a fn pointer, or joined with another
        // branch into one (it captures nothing): the value is its body
        // fn's trampoline, which supplies the empty env.
        let want = match dest {
            Some(d) if d.proj.is_empty() && matches!(self.kind(self.body.locals[d.local as usize].ty), TyKind::FnPtr(_)) => {
                self.body.locals[d.local as usize].ty
            }
            _ => ty,
        };
        if matches!(self.kind(want), TyKind::FnPtr(_))
            && info.captures.is_empty()
            && let Some(entry) = self.p.gx.closures.get(&info.def)
        {
            self.set_or_temp(dest, Rvalue::FnValue(entry.body_fn, want), want, span);
            return;
        }
        if info.captures.is_empty() {
            self.set_or_temp(dest, Rvalue::Use(Operand::Const(Const::Unit)), ty, span);
            return;
        }
        let mut ops = Vec::with_capacity(info.captures.len());
        for cap in &info.captures {
            let mut place = Place::local(self.local_map[cap.root as usize]);
            for pr in &cap.projs {
                place = place.project(*pr);
            }
            let op = match cap.mode {
                CaptureMode::Value if self.local_moved(cap.root) => Operand::Move(place),
                CaptureMode::Value => self.operand_from(place, cap.place_ty),
                m => {
                    let mutable = m != CaptureMode::Ref;
                    let rt = self.ref_ty(mutable, cap.place_ty);
                    let t = self.temp(rt);
                    self.assign(Place::local(t), Rvalue::Ref(mutable, place), span);
                    Operand::Move(Place::local(t))
                }
            };
            ops.push(op);
        }
        self.set_or_temp(dest, Rvalue::TupleAgg(ty, ops), ty, span);
    }

    /// A call of a closure value (`Res::ClosureCall`). The env operand
    /// follows the closure's own kind; the call's trait (`mode`) only
    /// adds a move: an `FnOnce` call of an `Fn` or `FnMut` closure value
    /// (not one behind a reference) moves it into a temporary and drops
    /// that right after the call, as `call_once` does.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn closure_call(&mut self, e: ExprId, callee: ExprId, args: &[ExprId], mode: ClosureKind, derefs: u8, dest: Option<&Place>, span: Span) {
        let ty = self.ty(e);
        let cty = self.peel(self.ty(callee), derefs);
        let TyKind::Closure(def, _) = self.kind(cty) else { return };
        let Some(entry) = self.p.gx.closures.get(&def) else { return };
        let (target, kind) = (entry.body_fn, entry.kind);
        // Through a reference (`F = &mut C` in an `F: FnOnce` instance),
        // an `FnOnce` call is core's `impl FnOnce for &F` or `&mut F`,
        // which calls the closure in place: it is never copied out.
        let by_ref = derefs > 0 && matches!(self.kind(self.ty(callee)), TyKind::Ref(..));
        let base = if mode == ClosureKind::FnMut || (by_ref && kind == ClosureKind::FnMut) { self.place_for_mut(callee) } else { self.place(callee) };
        let place = Self::derefs(base, derefs);
        let mut owned: Option<Local> = None;
        // A callee rustc moves where it is `Copy` here (`view_moved`). An
        // `FnOnce` call through a reference moves out of it (rustc E0507
        // on `let r = &h; r()` for an `impl FnOnce`).
        let moved = if derefs == 0 { self.view_moved(callee) } else { mode == ClosureKind::FnOnce };
        let env = match (kind, mode) {
            (ClosureKind::FnOnce, _) if moved => Operand::Move(place),
            (ClosureKind::FnOnce, _) => self.operand_from(place, cty),
            // (Behind a reference only an opaque `FnOnce` call moves: the
            // checker's view mode, rustc E0507 on `*r`.)
            (_, ClosureKind::FnOnce) if !by_ref || moved => {
                let own = self.temp(cty);
                let op = if moved { Operand::Move(place) } else { self.operand_from(place, cty) };
                self.assign(Place::local(own), Rvalue::Use(op), span);
                owned = Some(own);
                self.env_ref(Place::local(own), kind == ClosureKind::FnMut, cty, span)
            }
            (ClosureKind::FnMut, _) => self.env_ref(place, true, cty, span),
            // An `impl FnMut` call of an `Fn` closure borrows the opaque
            // value mutably (rustc E0596 on an immutable binding), then
            // passes the env shared.
            (ClosureKind::Fn, ClosureKind::FnMut) => {
                let Operand::Move(m) = self.env_ref(place, true, cty, span) else { unreachable!() };
                self.env_ref(m.deref(), false, cty, span)
            }
            (ClosureKind::Fn, _) => self.env_ref(place, false, cty, span),
        };
        let mut ops = vec![env];
        ops.extend(self.call_arg_operands(args, span));
        let d = self.call_dest(dest, ty);
        self.call_into(CallTarget::Fn(target), ops, d, span);
        if let Some(own) = owned {
            if needs_drop(self.p, &self.body.tys, cty) {
                self.push(StmtKind::Drop(own), span);
            }
        }
    }

    /// A reference to a closure value, as an env operand.
    fn env_ref(&mut self, place: Place, mutable: bool, cty: Ty, span: Span) -> Operand {
        let rt = self.ref_ty(mutable, cty);
        let t = self.temp(rt);
        self.assign(Place::local(t), Rvalue::Ref(mutable, place), span);
        Operand::Move(Place::local(t))
    }

    /// Rewrites every place rooted at a shadow local onto the env.
    fn rewrite_upvars(&mut self) {
        let Some(up) = self.upvars.take() else { return };
        let mut errs: Vec<Span> = Vec::new();
        let mut blocks = std::mem::take(&mut self.body.blocks);
        for b in blocks.iter_mut() {
            for st in b.stmts.iter_mut() {
                let span = st.span;
                let ok = match &mut st.kind {
                    StmtKind::Assign(pl, rv) => rewrite_place(&up, pl, false) & rewrite_rvalue(&up, rv),
                    StmtKind::Drop(l) | StmtKind::StorageDead(l) | StmtKind::FakeRead(l) => up.root_of(*l).is_none(),
                    StmtKind::Print { parts, .. } => rewrite_parts(&up, parts),
                };
                if !ok {
                    errs.push(span);
                }
            }
            let span = b.term_span;
            // An indirect call names its callee local: a captured fn
            // pointer is copied out of the env into a fresh temporary
            // first (mutable, as every temporary: a call in a loop
            // assigns it again).
            if let Terminator::Call { target: CallTarget::Indirect(l), .. } = &mut b.term
                && up.root_of(*l).is_some()
            {
                let mut pl = Place::local(*l);
                if rewrite_place(&up, &mut pl, true) {
                    let ty = self.body.locals[*l as usize].ty;
                    self.body.locals.push(LocalDecl { ty, mutable: true, name: None });
                    let t = self.body.locals.len() as Local - 1;
                    b.stmts.push(crate::ir::Stmt { kind: StmtKind::Assign(Place::local(t), Rvalue::Use(Operand::Copy(pl))), span });
                    *l = t;
                } else {
                    errs.push(span);
                }
            }
            let ok = match &mut b.term {
                Terminator::If(o, ..) | Terminator::Switch(o, ..) => rewrite_operand(&up, o),
                Terminator::Call { args, dest, .. } => args.iter_mut().fold(rewrite_place(&up, dest, false), |acc, o| acc & rewrite_operand(&up, o)),
                Terminator::Panic(parts) => rewrite_parts(&up, parts),
                Terminator::Goto(_) | Terminator::Return | Terminator::Unreachable | Terminator::MatchExhausted => true,
            };
            if !ok {
                errs.push(span);
            }
        }
        self.body.blocks = blocks;
        if let Some(&span) = errs.first() {
            self.body.lower_errs.push(Diag::new(
                "L0820",
                span,
                "closure lowering does not support this use of a captured variable yet",
                "Rewrite the use, or move the value into a local before the closure.",
            ));
        }
    }
}

/// Rewrites one place onto the env when it is rooted at a shadow local.
/// `read` is set for a copy of the place's value: a by-ref capture of
/// `P.*` then also covers a read of `P` itself (the reference value).
fn rewrite_place(up: &Upvars, pl: &mut Place, read: bool) -> bool {
    if pl.proj.iter().any(|pr| matches!(pr, Proj::Index(l) if up.root_of(*l).is_some())) {
        return false;
    }
    let Some(root) = up.root_of(pl.local) else { return true };
    for (i, cap) in up.info.captures.iter().enumerate() {
        if cap.root != root || cap.projs.len() > pl.proj.len() || cap.projs[..] != pl.proj[..cap.projs.len()] {
            continue;
        }
        let mut base = up.field[i].clone();
        if cap.mode != CaptureMode::Value {
            base = base.deref();
        }
        base.proj.extend_from_slice(&pl.proj[cap.projs.len()..]);
        *pl = base;
        return true;
    }
    if read {
        for (i, cap) in up.info.captures.iter().enumerate() {
            if cap.root == root
                && cap.mode == CaptureMode::Ref
                && cap.projs.len() == pl.proj.len() + 1
                && cap.projs[..pl.proj.len()] == pl.proj[..]
                && cap.projs.last() == Some(&Proj::Deref)
            {
                *pl = up.field[i].clone();
                return true;
            }
        }
    }
    false
}

fn rewrite_operand(up: &Upvars, o: &mut Operand) -> bool {
    match o {
        Operand::Copy(pl) => rewrite_place(up, pl, true),
        Operand::Move(pl) => rewrite_place(up, pl, false),
        Operand::Const(_) => true,
    }
}

fn rewrite_parts(up: &Upvars, parts: &mut [FmtPart]) -> bool {
    parts.iter_mut().fold(true, |acc, p| match p {
        FmtPart::Arg { value, .. } => acc & rewrite_operand(up, value),
        FmtPart::Lit(_) => acc,
    })
}

fn rewrite_rvalue(up: &Upvars, rv: &mut Rvalue) -> bool {
    match rv {
        Rvalue::Use(o) | Rvalue::Un(_, o) | Rvalue::Cast(o, _) | Rvalue::ArrayRepeat(_, o, _) | Rvalue::Res(_, _, o) => rewrite_operand(up, o),
        Rvalue::Opt(_, o) => o.as_mut().is_none_or(|o| rewrite_operand(up, o)),
        Rvalue::MakeDyn { ptr, .. } | Rvalue::DynUpcast { ptr, .. } | Rvalue::DynMeta { ptr, .. } | Rvalue::DynData { ptr, .. } => {
            rewrite_operand(up, ptr)
        }
        Rvalue::Bin(_, a, b)
        | Rvalue::StrCmp(_, a, b)
        | Rvalue::StringAdd(a, b)
        | Rvalue::VecRepeat(_, a, b)
        | Rvalue::Eq { a, b, .. }
        | Rvalue::Cmp { a, b, .. } => {
            rewrite_operand(up, a) & rewrite_operand(up, b)
        }
        Rvalue::Aggregate(_, _, ops) | Rvalue::VecLit(_, ops) | Rvalue::TupleAgg(_, ops) | Rvalue::ArrayAgg(_, ops) => {
            ops.iter_mut().fold(true, |acc, o| acc & rewrite_operand(up, o))
        }
        Rvalue::Format(parts) => rewrite_parts(up, parts),
        Rvalue::WriteFmt(dst, parts, _) | Rvalue::FormatArgs(Some(dst), parts) => rewrite_operand(up, dst) & rewrite_parts(up, parts),
        Rvalue::FormatArgs(None, parts) => rewrite_parts(up, parts),
        Rvalue::Ref(_, pl) | Rvalue::Discriminant(pl) | Rvalue::SliceOf(pl) | Rvalue::LenOf(pl) => rewrite_place(up, pl, false),
        Rvalue::Subslice { base, lo, hi, .. } => {
            rewrite_place(up, base, false) & rewrite_operand(up, lo) & hi.as_mut().is_none_or(|h| rewrite_operand(up, h))
        }
        Rvalue::StaticLoad(_) | Rvalue::StaticRef(_) | Rvalue::FnValue(..) | Rvalue::ItemShim(..) => true,
    }
}
