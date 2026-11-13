// Lowering of `core::any` operations (check/any.rs, `Res::Any`): type
// ids are `TypeId` aggregates of two `u64` constants, from the tested
// type (`check::any::type_id_words`) or from the two id words of a
// `dyn Any` vtable (`object::VT_ANY_ID`, reached through super slots for
// a user trait with `Any` as a supertrait). A downcast compares the two
// and, on a match, reads the dyn pointer's data pointer as the thin one
// (`Rvalue::DynData`).
use super::*;
use crate::check::any::{AnyOp, type_id_words};
use crate::check::object;

impl<'p> Lower<'p> {
    /// A `Res::Any` method call (`recv.type_id()`, `is`, the downcasts)
    /// into `dest`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn any_op_into(
        &mut self,
        e: ExprId,
        recv: ExprId,
        dest: Option<&Place>,
        span: Span,
        op: AnyOp,
        derefs: u8,
        autoref: Autoref,
    ) {
        let ty = self.ty(e);
        if op == AnyOp::Downcast {
            self.any_downcast_owned(e, recv, dest, ty, span);
            return;
        }
        // The receiver borrowed as the call's `&self` (`&mut self` for
        // `downcast_mut`), through any user `Deref` steps.
        let root = self.deref_steps(e, recv, span);
        let root_ty = match &root {
            Some(pl) => self.body.locals[pl.local as usize].ty,
            None => self.ty(recv),
        };
        let self_ty = self.peel(root_ty, derefs);
        let mutable = autoref == Autoref::RefMut;
        let pl = match root {
            Some(pl) => pl,
            None if mutable => self.place_for_mut(recv),
            None => self.place(recv),
        };
        let pl = Self::derefs(pl, derefs);
        let rt = self.ref_ty(mutable, self_ty);
        let r = self.temp(rt);
        self.assign(Place::local(r), Rvalue::Ref(mutable, pl), span);
        match op {
            AnyOp::TypeId => {
                let words = self.tested_words(e);
                self.type_id_into(dest, ty, words, span);
            }
            AnyOp::DynTypeId => {
                let (lo, hi) = self.dyn_id_words(Operand::Copy(Place::local(r)), self_ty, span);
                let out = self.call_dest(dest, ty);
                let adt = self.body.tys.local_adt(ty).expect("TypeId is a local Adt");
                self.assign(out, Rvalue::Aggregate(adt, 0, vec![lo, hi]), span);
            }
            AnyOp::Is => {
                let words = self.tested_words(e);
                let hit = self.id_test(Operand::Copy(Place::local(r)), self_ty, words, span);
                self.set_or_temp(dest, Rvalue::Use(hit), ty, span);
            }
            AnyOp::DowncastRef | AnyOp::DowncastMut => {
                let words = self.tested_words(e);
                let hit = self.id_test(Operand::Copy(Place::local(r)), self_ty, words, span);
                let Some(thin) = self.body.tys.option_elem(ty) else { return };
                let out = self.call_dest(dest, ty);
                let (yes, no, end) = (self.new_block(), self.new_block(), self.new_block());
                self.terminate(Terminator::If(hit, yes, no), span);
                self.cur = yes;
                let src = if mutable { Operand::Move(Place::local(r)) } else { Operand::Copy(Place::local(r)) };
                let t = self.temp(thin);
                self.assign(Place::local(t), Rvalue::DynData { ptr: src, ty: thin }, span);
                let moved = if mutable { Operand::Move(Place::local(t)) } else { Operand::Copy(Place::local(t)) };
                self.assign(out.clone(), Rvalue::Opt(ty, Some(moved)), span);
                self.goto(end, span);
                self.cur = no;
                self.assign(out, Rvalue::Opt(ty, None), span);
                self.goto(end, span);
                self.cur = end;
            }
            AnyOp::Downcast | AnyOp::TypeIdOf | AnyOp::TypeName => unreachable!("not a by-reference `Any` method"),
        }
    }

    /// `Box<dyn Any>::downcast::<T>()` or `Rc<dyn Any>::downcast::<T>()`:
    /// the receiver moves into `Ok` as `Box<T>` (`Rc<T>`) when its type id
    /// is T's, else into `Err` unchanged.
    fn any_downcast_owned(&mut self, e: ExprId, recv: ExprId, dest: Option<&Place>, ty: Ty, span: Span) {
        let recv_ty = self.ty(recv);
        let b = self.temp(recv_ty);
        self.expr_into(recv, Some(&Place::local(b)));
        let Some(dyn_ty) = self.body.tys.smart_elem(recv_ty) else { return };
        let Some((ok_ty, _)) = self.body.tys.result_args(ty) else { return };
        let words = self.tested_words(e);
        let hit = self.id_test(Operand::Copy(Place::local(b)), dyn_ty, words, span);
        let out = self.call_dest(dest, ty);
        let (yes, no, end) = (self.new_block(), self.new_block(), self.new_block());
        self.terminate(Terminator::If(hit, yes, no), span);
        self.cur = yes;
        let t = self.temp(ok_ty);
        self.assign(Place::local(t), Rvalue::DynData { ptr: Operand::Move(Place::local(b)), ty: ok_ty }, span);
        self.assign(out.clone(), Rvalue::Res(ty, true, Operand::Move(Place::local(t))), span);
        self.goto(end, span);
        self.cur = no;
        self.assign(out, Rvalue::Res(ty, false, Operand::Move(Place::local(b))), span);
        self.goto(end, span);
        self.cur = end;
    }

    /// `std::any::type_name::<T>()` into `dest`: rustc's exact
    /// string for the type check recorded, baked as a `&str`
    /// constant. Check rejected every type it cannot print.
    pub(super) fn type_name_into(&mut self, e: ExprId, dest: Option<&Place>, ty: Ty, span: Span) {
        let t = self.ck.generic_args.get((e.0 - self.ck.expr_base) as usize).and_then(|a| a.first()).and_then(|g| g.as_ty());
        let t = t.expect("check records the named type of every `type_name` call");
        // A template body lowers with its parameters still open:
        // the string is a placeholder there (each instance bakes
        // its own), as a layout query's zero is. So is a rejected
        // body's (check reported the type; it is never generated).
        let open = self.body.tys.flags(t).contains_param() || self.body.tys.flags(t).contains_projection();
        let s = if open {
            String::new()
        } else {
            crate::check::any::type_name_of(self.p, &mut self.body.tys, t, false).unwrap_or_default()
        };
        let blob = crate::ast::ByteBlob::new(s.into_bytes());
        self.set_or_temp(dest, Rvalue::Use(Operand::Const(Const::StrBytes(blob))), ty, span);
    }

    /// `TypeId::of::<T>()` (the call's resolution) into `dest`.
    pub(super) fn type_id_of_into(&mut self, e: ExprId, dest: Option<&Place>, ty: Ty, span: Span) {
        let words = self.tested_words(e);
        self.type_id_into(dest, ty, words, span);
    }

    /// The id words of the type check recorded for `e`.
    fn tested_words(&self, e: ExprId) -> (u64, u64) {
        let t = self.ck.generic_args.get((e.0 - self.ck.expr_base) as usize).and_then(|a| a.first()).and_then(|g| g.as_ty());
        let t = t.expect("check records the tested type of every `Res::Any`");
        type_id_words(self.p, &self.body.tys, t)
    }

    /// A `TypeId` value of the two words into `dest`.
    fn type_id_into(&mut self, dest: Option<&Place>, ty: Ty, (lo, hi): (u64, u64), span: Span) {
        let adt = self.body.tys.local_adt(ty).expect("TypeId is a local Adt");
        let w = |v: u64| Operand::Const(Const::Int(v as u128, IntTy::U64));
        self.set_or_temp(dest, Rvalue::Aggregate(adt, 0, vec![w(lo), w(hi)]), ty, span);
    }

    /// The two type id words of the value behind `ptr`, a pointer to
    /// `dyn_ty` (whose principal reaches `Any`): upcast to `dyn Any`
    /// first when the principal is a user trait.
    fn dyn_id_words(&mut self, ptr: Operand, dyn_ty: Ty, span: Span) -> (Operand, Operand) {
        let any = self.p.gx.lang.any;
        let ptr = match object::principal_of(&self.body.tys, dyn_ty) {
            Some(pr) if pr.def != any => {
                let args = self.body.tys.intern_args(&[GenericArg::Ty(DYN_SELF)]);
                let to = TraitRef { def: any, args, bindings: EMPTY_BINDINGS };
                let path = object::upcast_path(self.p, &mut self.body.tys, pr, to).expect("check found `Any` among the supertraits");
                let bounds = self.body.tys.intern_dyn_bounds(DynBounds {
                    principal: Some(Binder::plain(to)),
                    supertraits: Box::new([]),
                    auto: AutoTraits::default(),
                });
                let any_dyn = self.body.tys.intern(TyKind::Dyn(bounds, Region::Erased));
                let rt = self.ref_ty(false, any_dyn);
                let t = self.temp(rt);
                self.assign(Place::local(t), Rvalue::DynUpcast { ptr, path }, span);
                Operand::Copy(Place::local(t))
            }
            _ => ptr,
        };
        let u64_ty = int(IntTy::U64);
        let mut word = |this: &mut Self, k: u32| {
            let t = this.temp(u64_ty);
            this.assign(Place::local(t), Rvalue::DynMeta { ptr: ptr.clone(), word: object::VT_ANY_ID + k }, span);
            Operand::Copy(Place::local(t))
        };
        let lo = word(self, 0);
        let hi = word(self, 1);
        (lo, hi)
    }

    /// True (a `bool` operand) when the value behind `ptr` (a pointer to
    /// `dyn_ty`) has the type id `words`.
    fn id_test(&mut self, ptr: Operand, dyn_ty: Ty, (lo, hi): (u64, u64), span: Span) -> Operand {
        let (a, b) = self.dyn_id_words(ptr, dyn_ty, span);
        let w = |v: u64| Operand::Const(Const::Int(v as u128, IntTy::U64));
        let (x, y, both) = (self.temp(BOOL), self.temp(BOOL), self.temp(BOOL));
        self.assign(Place::local(x), Rvalue::Bin(BinOp::Eq, a, w(lo)), span);
        self.assign(Place::local(y), Rvalue::Bin(BinOp::Eq, b, w(hi)), span);
        self.assign(Place::local(both), Rvalue::Bin(BinOp::BitAnd, Operand::Copy(Place::local(x)), Operand::Copy(Place::local(y))), span);
        Operand::Copy(Place::local(both))
    }
}
