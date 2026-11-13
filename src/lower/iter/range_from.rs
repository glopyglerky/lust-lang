//! `RangeFrom` as an iterator (`(0..).zip(v)`, `for i in 1u8..`):
//! core 1.99's `Iterator for ops::RangeFrom<A>` over `Step::forward`.
//!
//! `next` computes `forward(start, 1)` before it returns the old start,
//! and `nth(n)` is `plus = forward(start, n); start = forward(plus, 1)`,
//! so the step past the last value panics before that value comes out.
//! An integer `forward` with no room left runs `Self::MAX + 1` at the
//! `step_integer_impls!` line (core/src/iter/range.rs:548:1), which
//! panics with overflow checks on (`#[rustc_inherit_overflow_checks]`);
//! the step itself is a wrapping add. A char `forward` past `char::MAX`
//! is the trait default's `expect` (range.rs:118:45) in every build;
//! chars skip the surrogates (char_range.rs step indexes).

use super::*;

impl Ex<'_, '_> {
    /// `Step::forward(start, n)` for the integer or char element `elem`,
    /// as a new local.
    fn step_forward(&mut self, start: Operand, n: Local, elem: Ty) -> Result<Local, String> {
        use crate::ast::BinOp::*;
        let w = int(IntTy::I128);
        if matches!(self.body.tys.kind(elem), TyKind::Char) {
            let idx = self.char_idx(start);
            let wi = self.cast_local(Operand::Copy(Place::local(idx)), w);
            let wn = self.cast_local(Operand::Copy(Place::local(n)), w);
            let plus = self.wide_bin(false, wi, wn, w);
            let over = self.cmp(Gt, Operand::Copy(Place::local(plus)), Operand::Const(Const::Int(u128::from(char_range::CMAX), IntTy::I128)));
            let (bad, ok) = self.branch(over);
            self.cur = bad;
            self.term(Terminator::Panic(vec![FmtPart::Lit("overflow in `Step::forward`".into())]));
            self.body.blocks[self.cur.0 as usize].term_span = Span::std_site(8, 118, 45);
            self.cur = ok;
            let i2 = self.cast_local(Operand::Copy(Place::local(plus)), int(IntTy::U32));
            return Ok(self.char_of_idx(Operand::Copy(Place::local(i2))));
        }
        let (w, it) = self.wide_ty(elem)?;
        let ws = self.cast_local(start, w);
        let wn = self.cast_local(Operand::Copy(Place::local(n)), w);
        let maxw = self.wide_bound(it, w, true);
        let room = self.wide_bin(true, maxw, ws, w);
        let over = self.cmp(Gt, Operand::Copy(Place::local(wn)), Operand::Copy(Place::local(room)));
        let (bad, ok) = self.branch(over);
        self.cur = bad;
        // `let _ = Self::MAX + 1;`: an overflow panic only when the
        // build checks overflow.
        let max_v: u128 = if it.signed() { (1u128 << (it.bits() - 1)) - 1 } else if it.bits() == 128 { u128::MAX } else { (1u128 << it.bits()) - 1 };
        let dead = self.local(elem);
        let at = std::mem::replace(&mut self.span, Span::std_site(8, 548, 1));
        self.assign(Place::local(dead), Rvalue::Bin(Add, Operand::Const(Const::Int(max_v, it)), Operand::Const(Const::Int(1, it))));
        self.span = at;
        self.goto(ok);
        self.cur = ok;
        let plus = self.wide_bin(false, ws, wn, w);
        Ok(self.cast_local(Operand::Copy(Place::local(plus)), elem))
    }

    /// `next` of the `RangeFrom` at `itp` (element `elem`) into `dest`.
    pub(super) fn range_from_next(&mut self, itp: Place, elem: Ty, dest: Place) -> R {
        let opt = self.body.tys.mk_option(elem);
        let start = itp.project(Proj::Field(0));
        let one = self.copy_local(Self::usize_c(1), USIZE);
        let n = self.step_forward(Operand::Copy(start.clone()), one, elem)?;
        let old = self.copy_local(Operand::Copy(start.clone()), elem);
        self.assign(start, Rvalue::Use(Operand::Copy(Place::local(n))));
        self.some(&dest, opt, Operand::Copy(Place::local(old)));
        Ok(())
    }

    /// `nth(n)` of the `RangeFrom` at `itp` into `dest`.
    pub(super) fn range_from_nth(&mut self, itp: Place, elem: Ty, n: Local, dest: Place) -> R {
        let opt = self.body.tys.mk_option(elem);
        let start = itp.project(Proj::Field(0));
        let plus = self.step_forward(Operand::Copy(start.clone()), n, elem)?;
        let one = self.copy_local(Self::usize_c(1), USIZE);
        let next = self.step_forward(Operand::Copy(Place::local(plus)), one, elem)?;
        self.assign(start, Rvalue::Use(Operand::Copy(Place::local(next))));
        self.some(&dest, opt, Operand::Copy(Place::local(plus)));
        Ok(())
    }
}
