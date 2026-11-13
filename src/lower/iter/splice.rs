//! `v.splice(range, items);` as a statement (R4): the `Splice` drops at
//! once, so the call is alloc's drain of the range, then `Splice::drop`:
//! the drained items drop front to back, then the replacement fills the
//! hole. With no tail it extends the Vec (`extend_trusted`: one `reserve`
//! of the exact length); else it fills the hole, moves the tail by the
//! size hint's lower bound (`move_tail`: `reserve(tail_start + tail_len,
//! n)`), and fills again. The replacement is a `TrustedLen` std iterator
//! (check/slice.rs `splice_trusted`), so its hint is exact and nothing is
//! left to collect. The Drain's drop then moves the tail down and sets
//! the length. The steps on the Drain are runtime builtins (cgen/slices.rs).

use super::*;

impl Ex<'_, '_> {
    /// The iterator a `splice` replacement value turns into
    /// (`IntoIterator`), for the forms check takes.
    fn splice_iter_ty(&mut self, t: Ty) -> Result<Ty, String> {
        use IterKind as K;
        let tys = &mut self.body.tys;
        if it::is_iter(tys, t) {
            return Ok(t);
        }
        if let Some(el) = tys.vec_elem(t) {
            return Ok(it::mk_iter(tys, K::VecIntoIter, &[el]));
        }
        match tys.kind(t) {
            TyKind::Array(el, _) => Ok(it::mk_iter(tys, K::ArrayIntoIter, &[el, t])),
            TyKind::Ref(_, m, inner) => {
                let el = match tys.kind(inner) {
                    TyKind::Slice(el) | TyKind::Array(el, _) => Some(el),
                    _ => tys.vec_elem(inner),
                }
                .ok_or("splice expansion: replacement is not a sequence")?;
                Ok(it::mk_iter(tys, if m == Mut::Mut { K::SliceIterMut } else { K::SliceIter }, &[el]))
            }
            _ => Err("splice expansion: replacement is not an iterator".into()),
        }
    }

    /// The `splice` expansion: `v` is the Vec's `&mut`, `bounds` drain's
    /// four range words, `rw` the replacement value.
    pub(super) fn splice(&mut self, v: Operand, bounds: [Operand; 4], rw: Operand) -> R {
        use crate::ast::BinOp::{Eq, Gt};
        let vt = self.op_ty(&v)?;
        let elem = match self.body.tys.kind(vt) {
            TyKind::Ref(_, _, vv) => self.body.tys.vec_elem(vv),
            _ => None,
        }
        .ok_or("splice expansion: receiver is not a Vec reference")?;
        let dty = it::mk_iter(&mut self.body.tys, IterKind::VecDrain, &[elem]);
        let d = self.local(dty);
        let [st, sv, et, ev] = bounds;
        self.call_builtin(Builtin::VecDrain, vec![v, st, sv, et, ev], Place::local(d));
        let rwt = self.op_ty(&rw)?;
        let ity = self.splice_iter_ty(rwt)?;
        let iv = self.into_iter(rw, ity)?;
        let itl = self.copy_local(iv, ity);
        let u = self.local(UNIT);
        let tail = self.local(USIZE);
        let dr = self.ref_to(true, Place::local(d), dty);
        self.call_builtin(Builtin::SpliceBegin, vec![Operand::Move(Place::local(dr))], Place::local(tail));
        let done = self.block();
        let no_tail = self.cmp(Eq, Operand::Copy(Place::local(tail)), Self::usize_c(0));
        let (ext, fill) = self.branch(no_tail);
        // No tail: `vec.extend(replace_with.by_ref())`.
        self.cur = ext;
        let (lo, _) = self.hint(Place::local(itl), ity)?;
        let dr = self.ref_to(true, Place::local(d), dty);
        self.call_builtin(Builtin::SpliceReserve, vec![Operand::Move(Place::local(dr)), Operand::Copy(Place::local(lo))], Place::local(u));
        self.splice_fill(d, dty, itl, ity, elem, false, done)?;
        // A tail: fill the hole, then move the tail by the hint and fill.
        self.cur = fill;
        let after = self.splice_fill(d, dty, itl, ity, elem, true, done)?;
        self.cur = after;
        let (lo, _) = self.hint(Place::local(itl), ity)?;
        let more = self.cmp(Gt, Operand::Copy(Place::local(lo)), Self::usize_c(0));
        let (mv, skip) = self.branch(more);
        self.cur = skip;
        self.goto(done);
        self.cur = mv;
        let dr = self.ref_to(true, Place::local(d), dty);
        self.call_builtin(Builtin::SpliceMoveTail, vec![Operand::Move(Place::local(dr)), Operand::Copy(Place::local(lo))], Place::local(u));
        let after2 = self.splice_fill(d, dty, itl, ity, elem, true, done)?;
        self.cur = after2;
        self.goto(done);
        self.cur = done;
        self.drop_local(d);
        self.drop_local(itl);
        Ok(())
    }

    /// Pulls items into the Drain's hole: while there is a gap (or,
    /// for the extend form, always), `next` and put. An end of the
    /// replacement goes to `done`; with `gap`, the block after the
    /// hole is full is returned (the extend form never leaves but by
    /// `done`; its return block is unreachable).
    #[allow(clippy::too_many_arguments)]
    fn splice_fill(&mut self, d: Local, dty: Ty, itl: Local, ity: Ty, elem: Ty, gap: bool, done: Bb) -> Result<Bb, String> {
        let lp = self.block();
        let full = self.block();
        self.goto(lp);
        self.cur = lp;
        if gap {
            let g = self.local(BOOL);
            let dr = self.ref_to(false, Place::local(d), dty);
            self.call_builtin(Builtin::SpliceGap, vec![Operand::Move(Place::local(dr))], Place::local(g));
            let (pull, closed) = self.branch(Operand::Copy(Place::local(g)));
            self.cur = closed;
            self.goto(full);
            self.cur = pull;
        }
        let opt = self.body.tys.mk_option(elem);
        let t = self.local(opt);
        self.next(Place::local(itl), ity, Place::local(t), false)?;
        let (s, n) = self.switch_opt(&Place::local(t));
        self.cur = n;
        self.goto(done);
        self.cur = s;
        let x = self.take(Self::payload(&Place::local(t)), elem);
        let u = self.local(UNIT);
        let dr = self.ref_to(true, Place::local(d), dty);
        self.call_builtin(Builtin::SplicePut, vec![Operand::Move(Place::local(dr)), x], Place::local(u));
        self.goto(lp);
        Ok(full)
    }
}
