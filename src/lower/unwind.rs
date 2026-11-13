// Lowering of `std::panic`'s compiler-known fns (W-UNWIND, docs/ARCH.md
// section 7.2; typed in check/unwind.rs):
//
// - `catch_unwind(f)` is the catch boundary: its call has no cleanup
//   successor, so a panic inside `f` comes back to it, and codegen turns
//   the exception into the `Err` value (cgen/unwind.rs). An
//   `AssertUnwindSafe(f)` argument passes its `f`.
// - `panic_any(m)` boxes `m` as a `Box<dyn Any + Send>` first; the
//   runtime then raises that payload.
// - `resume_unwind(payload)` is a plain builtin call.
use super::*;
use crate::check::sync::SyncOp;

impl<'p> Lower<'p> {
    /// Lowers one `std::panic` builtin call with lowered arguments `ops`.
    pub(super) fn unwind_call(&mut self, op: SyncOp, args: &[ExprId], mut ops: Vec<Operand>, dest: Option<&Place>, ty: Ty, span: Span) {
        let target = CallTarget::Builtin(Builtin::Sync(op));
        match op {
            SyncOp::CatchUnwind => {
                let at = self.ty(args[0]);
                if let Some(inner) = self.assert_unwind_safe_inner(at)
                    && let Some(Operand::Move(pl) | Operand::Copy(pl)) = ops.first().cloned()
                {
                    let t = self.temp(inner);
                    let f = self.operand_from(pl.project(Proj::Field(0)), inner);
                    self.assign(Place::local(t), Rvalue::Use(f), span);
                    ops[0] = Operand::Move(Place::local(t));
                }
                let d = self.call_dest(dest, ty);
                self.call_no_unwind(target, ops, d, span);
            }
            SyncOp::PanicAny => {
                let m = self.ty(args[0]);
                let boxed = self.body.tys.mk_boxed(m);
                let b = self.temp(boxed);
                self.call_into(CallTarget::Builtin(Builtin::BoxNew), ops, Place::local(b), span);
                let dyn_ty = crate::check::unwind::dyn_any_send(&mut self.body.tys, self.p.gx.lang.any);
                let dyn_box = self.body.tys.mk_boxed(dyn_ty);
                let p = self.temp(dyn_box);
                self.assign(Place::local(p), Rvalue::MakeDyn { ptr: Operand::Move(Place::local(b)), src: m, dyn_ty }, span);
                let d = self.call_dest(dest, ty);
                self.call_into(target, vec![Operand::Move(Place::local(p))], d, span);
            }
            _ => {
                let d = self.call_dest(dest, ty);
                self.call_into(target, ops, d, span);
            }
        }
    }

    /// The `F` of an `AssertUnwindSafe<F>` type.
    fn assert_unwind_safe_inner(&self, t: Ty) -> Option<Ty> {
        let a = self.body.tys.local_adt(t)?;
        if !crate::check::unwind::is_assert_unwind_safe(self.p, a) {
            return None;
        }
        self.body.tys.adt_first_ty_arg(t)
    }

    /// A call with no cleanup successor (a catch boundary): codegen does
    /// not check it, and handles a panic inside it itself.
    fn call_no_unwind(&mut self, target: CallTarget, args: Vec<Operand>, dest: Place, span: Span) {
        let next = self.new_block();
        let b = &mut self.body.blocks[self.cur.0 as usize];
        b.term = Terminator::Call { target, args, dest, next, unwind: None };
        b.term_span = span;
        self.cur = next;
    }
}
