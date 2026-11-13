//! `Range<char>` and `RangeInclusive<char>` steps: core's `Step for
//! char`, which skips the surrogates `0xD800..0xE000`.
//!
//! A char maps to its step index (chars from `0xE000` move down by
//! `0x800`), so char steps are `u32` steps on indices `0..=CMAX`. Most
//! operations run the `u32` range expansion on an index copy of the
//! range and store the copy's bounds back as chars. Core's char
//! `forward_overflowing` and `backward_overflowing` saturate at
//! `char::MAX` and `'\0'` where the integer forms wrap, so the inclusive
//! `nth` and `nth_back` have their own expansion here.

use super::*;

/// The step index of `char::MAX`.
pub(super) const CMAX: u32 = 0x10F7FF;

impl Ex<'_, '_> {
    /// True for `Range<char>` and `RangeInclusive<char>`.
    pub(super) fn is_char_range(&self, ty: Ty) -> bool {
        matches!(self.kind_args(ty), Some((IterKind::Range | IterKind::RangeInclusive, a)) if matches!(self.body.tys.kind(a[0]), TyKind::Char))
    }

    /// A new `u32` local: the step index of a char.
    pub(super) fn char_idx(&mut self, c: Operand) -> Local {
        let u32t = int(IntTy::U32);
        let v = self.cast_local(c, u32t);
        let hi = self.cmp(crate::ast::BinOp::Ge, Operand::Copy(Place::local(v)), Self::u32_c(0xE000));
        let (down, keep) = self.branch(hi);
        let join = self.block();
        self.cur = down;
        self.assign(Place::local(v), Rvalue::Bin(crate::ast::BinOp::Sub, Operand::Copy(Place::local(v)), Self::u32_c(0x800)));
        self.goto(join);
        self.cur = keep;
        self.goto(join);
        self.cur = join;
        v
    }

    /// A new `char` local: the char of a `u32` step index.
    pub(super) fn char_of_idx(&mut self, i: Operand) -> Local {
        let v = self.copy_local(i, int(IntTy::U32));
        let hi = self.cmp(crate::ast::BinOp::Ge, Operand::Copy(Place::local(v)), Self::u32_c(0xD800));
        let (up, keep) = self.branch(hi);
        let join = self.block();
        self.cur = up;
        self.assign(Place::local(v), Rvalue::Bin(crate::ast::BinOp::Add, Operand::Copy(Place::local(v)), Self::u32_c(0x800)));
        self.goto(join);
        self.cur = keep;
        self.goto(join);
        self.cur = join;
        self.cast_local(Operand::Copy(Place::local(v)), CHAR)
    }

    /// Runs `op` on a `u32` index copy of the char range at `itp` (the
    /// copy's place and type), then stores the copy back. An inclusive
    /// bound past `CMAX` is a step that overflowed: core leaves the
    /// bound at `char::MAX` (or `'\0'` for `end`) and the range
    /// exhausted.
    fn char_range_via_idx<T>(&mut self, itp: &Place, ty: Ty, op: impl FnOnce(&mut Self, Place, Ty) -> Result<T, String>) -> Result<T, String> {
        use crate::ast::BinOp::*;
        let (k, _) = self.kind_args(ty).ok_or("iterator expansion: char range")?;
        let incl = k == IterKind::RangeInclusive;
        let u32t = int(IntTy::U32);
        let sty = it::mk_iter(&mut self.body.tys, k, &[u32t]);
        let sh = Place::local(self.local(sty));
        let f = |p: &Place, i: u32| p.project(Proj::Field(i));
        for i in 0..2 {
            let x = self.char_idx(Operand::Copy(f(itp, i)));
            self.assign(f(&sh, i), Rvalue::Use(Operand::Copy(Place::local(x))));
        }
        if incl {
            self.assign(f(&sh, 2), Rvalue::Use(Operand::Copy(f(itp, 2))));
        }
        let r = op(self, sh.clone(), sty)?;
        if incl {
            for (i, sat) in [(0, CMAX), (1, 0)] {
                let over = self.cmp(Gt, Operand::Copy(f(&sh, i)), Self::u32_c(CMAX));
                let (fix, ok) = self.branch(over);
                let join = self.block();
                self.cur = fix;
                self.assign(f(&sh, i), Rvalue::Use(Self::u32_c(sat)));
                self.assign(f(&sh, 2), Rvalue::Use(Operand::Const(Const::Bool(true))));
                self.goto(join);
                self.cur = ok;
                self.goto(join);
                self.cur = join;
            }
            self.assign(f(itp, 2), Rvalue::Use(Operand::Copy(f(&sh, 2))));
        }
        for i in 0..2 {
            let c = self.char_of_idx(Operand::Copy(f(&sh, i)));
            self.assign(f(itp, i), Rvalue::Use(Operand::Copy(Place::local(c))));
        }
        Ok(r)
    }

    /// `dest = src` mapped from an `Option<u32>` of step indices to an
    /// `Option<char>`.
    fn char_opt_of_idx(&mut self, src: Local, dest: &Place) {
        let opt = self.body.tys.mk_option(CHAR);
        let (s, n) = self.switch_opt(&Place::local(src));
        let join = self.block();
        self.cur = s;
        let c = self.char_of_idx(Operand::Copy(Self::payload(&Place::local(src))));
        self.some(dest, opt, Operand::Copy(Place::local(c)));
        self.goto(join);
        self.cur = n;
        self.none(dest, opt);
        self.goto(join);
        self.cur = join;
    }

    /// `dest = itp.next()` (or `next_back`) on a char range.
    pub(super) fn char_range_next(&mut self, itp: Place, ty: Ty, dest: Place, back: bool) -> R {
        let ou = self.body.tys.mk_option(int(IntTy::U32));
        let t = self.local(ou);
        self.char_range_via_idx(&itp, ty, |this, sh, sty| this.next(sh, sty, Place::local(t), back))?;
        self.char_opt_of_idx(t, &dest);
        Ok(())
    }

    /// `dest = itp.nth(n)` (or `nth_back`) on a char range.
    pub(super) fn char_range_nth(&mut self, itp: Place, ty: Ty, n: Local, dest: Place, back: bool) -> R {
        if matches!(self.kind_args(ty), Some((IterKind::RangeInclusive, _))) {
            return self.char_incl_nth(itp, n, dest, back);
        }
        let ou = self.body.tys.mk_option(int(IntTy::U32));
        let t = self.local(ou);
        self.char_range_via_idx(&itp, ty, |this, sh, sty| {
            if back { this.nth_back(sh, sty, n, Place::local(t)) } else { this.nth(sh, sty, n, Place::local(t)) }
        })?;
        self.char_opt_of_idx(t, &dest);
        Ok(())
    }

    /// `itp.advance_by(n)` (or `advance_back_by`) on a char range: the
    /// remainder local.
    pub(super) fn char_range_advance(&mut self, itp: Place, ty: Ty, n: Local, back: bool) -> Result<Local, String> {
        self.char_range_via_idx(&itp, ty, |this, sh, sty| if back { this.advance_back(sh, sty, n) } else { this.advance(sh, sty, n) })
    }

    /// `itp.size_hint()` on a char range (see `hint`).
    pub(super) fn char_range_hint(&mut self, itp: Place, ty: Ty) -> Result<(Local, Local), String> {
        self.char_range_via_idx(&itp, ty, |this, sh, sty| this.hint(sh, sty))
    }

    /// The exact length of a char `Range` (see `size`).
    pub(super) fn char_range_size(&mut self, itp: Place, ty: Ty) -> Result<Operand, String> {
        self.char_range_via_idx(&itp, ty, |this, sh, sty| this.size(sh, sty))
    }

    /// `dest = itp.count()` on a char range.
    pub(super) fn char_range_count(&mut self, itp: Place, ty: Ty, dest: Place) -> R {
        self.char_range_via_idx(&itp, ty, |this, sh, sty| this.count(sh, sty, dest))
    }

    /// `RangeInclusive<char>::nth` (or `nth_back`): core's generic form
    /// over char's saturating overflowing steps. Forward:
    /// `(plus_n, on) = forward_overflowing(start, n)`, `(plus_1, o1) =
    /// forward_overflowing(plus_n, 1)`, `start = plus_1`, `exhausted =
    /// on | o1`, and `Some(plus_n)` when `!on && plus_n <= end`. An
    /// overflow saturates at `CMAX`, so `on | o1` is `plus_n == CMAX`.
    fn char_incl_nth(&mut self, itp: Place, n: Local, dest: Place, back: bool) -> R {
        use crate::ast::BinOp::*;
        let opt = self.body.tys.mk_option(CHAR);
        let f = |i: u32| itp.project(Proj::Field(i));
        let w = int(IntTy::I128);
        let wc = |v: u32| Operand::Const(Const::Int(v as u128, IntTy::I128));
        let join = self.block();
        let none_b = self.block();
        // `is_empty`: exhausted, or not `start <= end`.
        let (ex, live) = self.branch(Operand::Copy(f(2)));
        self.cur = ex;
        self.goto(none_b);
        self.cur = live;
        let c = self.cmp(Le, Operand::Copy(f(0)), Operand::Copy(f(1)));
        let (go, empty) = self.branch(c);
        self.cur = empty;
        self.goto(none_b);
        self.cur = go;
        let (from, other) = if back { (1, 0) } else { (0, 1) };
        let fi = self.char_idx(Operand::Copy(f(from)));
        let wf = self.cast_local(Operand::Copy(Place::local(fi)), w);
        let wn = self.cast_local(Operand::Copy(Place::local(n)), w);
        let at = self.wide_bin(back, wf, wn, w);
        // `on`: the n-step overflowed; then `at` saturates.
        let (limit, on) = if back {
            (0, self.cmp(Lt, Operand::Copy(Place::local(at)), wc(0)))
        } else {
            (CMAX, self.cmp(Gt, Operand::Copy(Place::local(at)), wc(CMAX)))
        };
        let (sat, fits) = self.branch(on.clone());
        let next_b = self.block();
        self.cur = sat;
        self.assign(Place::local(at), Rvalue::Use(wc(limit)));
        self.goto(next_b);
        self.cur = fits;
        self.goto(next_b);
        self.cur = next_b;
        // The one more step, saturating: the new bound and `exhausted`.
        let o1 = self.cmp(Eq, Operand::Copy(Place::local(at)), wc(limit));
        let step = self.local(w);
        self.assign(Place::local(step), Rvalue::Use(Operand::Copy(Place::local(at))));
        let (stay, mv) = self.branch(o1.clone());
        let set_b = self.block();
        self.cur = mv;
        let op = if back { Sub } else { Add };
        self.assign(Place::local(step), Rvalue::Bin(op, Operand::Copy(Place::local(step)), wc(1)));
        self.goto(set_b);
        self.cur = stay;
        self.goto(set_b);
        self.cur = set_b;
        let si = self.cast_local(Operand::Copy(Place::local(step)), int(IntTy::U32));
        let sc = self.char_of_idx(Operand::Copy(Place::local(si)));
        self.assign(f(from), Rvalue::Use(Operand::Copy(Place::local(sc))));
        self.assign(f(2), Rvalue::Use(o1));
        // The item: `!on` and within the other bound.
        let (ovf, in_b) = self.branch(on);
        self.cur = ovf;
        self.goto(none_b);
        self.cur = in_b;
        let oi = self.char_idx(Operand::Copy(f(other)));
        let wo = self.cast_local(Operand::Copy(Place::local(oi)), w);
        let ok = if back {
            self.cmp(Ge, Operand::Copy(Place::local(at)), Operand::Copy(Place::local(wo)))
        } else {
            self.cmp(Le, Operand::Copy(Place::local(at)), Operand::Copy(Place::local(wo)))
        };
        let (hit, miss) = self.branch(ok);
        self.cur = miss;
        self.goto(none_b);
        self.cur = hit;
        let ai = self.cast_local(Operand::Copy(Place::local(at)), int(IntTy::U32));
        let ac = self.char_of_idx(Operand::Copy(Place::local(ai)));
        self.some(&dest, opt, Operand::Copy(Place::local(ac)));
        self.goto(join);
        self.cur = none_b;
        self.none(&dest, opt);
        self.goto(join);
        self.cur = join;
        Ok(())
    }
}
