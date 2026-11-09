//! Match exhaustiveness by rustc's usefulness algorithm. The arms form a
//! matrix of pattern rows; the check specializes it one constructor at a
//! time, and a constructor no row names sends it to the default matrix
//! (the rows whose head is a wildcard). Integer and char ranges and slice
//! lengths are split into the few constructors the rows can tell apart,
//! as rustc does, so the check stays exact without listing every value.
//!
//! As in rustc, `usize` and `isize` ranges never reach the ends of their
//! domain: only a half-open `X..` (or `..X` for `isize`) covers the values
//! past `MAX` (and before `MIN`).
//!
//! A structural const pattern (a struct, enum, tuple, array, or slice
//! const) is read as the patterns of its value, as rustc's
//! const-to-pattern does: its parts become `Cell::Val` cells.
//!
//! Where lust cannot read a pattern (a const of unknown value, a byte
//! string, a float or `&str` part of a const) the row matches nothing, and a type whose values cannot be
//! listed (floats, `str`, generics) needs a wildcard. Both can only make
//! lust reject a match that rustc accepts, never the reverse.

use super::*;

/// A row cell.
#[derive(Clone, Copy)]
enum Cell<'p> {
    /// A wildcard made by specialization.
    Wild,
    Pat(PatId),
    /// A part of a structural const pattern's value, at its type.
    Val(&'p crate::ty::consteval::ConstValue, Ty),
}

/// A point in the extended integer order: `(0, 0)` is below `MIN`
/// (only `isize` reaches it), `(1, k)` is the value whose order key
/// (`Ctx::scalar_key`) is `k`, `(2, 0)` is above `MAX` (reached only
/// through `usize` and `isize` half-open ranges), and `(3, 0)` is past
/// everything. Ranges are half-open: `[lo, hi)`.
type Bound = (u8, u128);

/// The constructors of a column's type.
enum Col {
    /// A fixed list of constructors, by index, with each one's field
    /// count and whether the constructor is empty (a field of an
    /// uninhabited type, so no value uses it): enum variants,
    /// `Option` (`None`, `Some`), `Result` (`Ok`, `Err`), `bool`
    /// (`false`, `true`), and the one constructor of a tuple or
    /// struct.
    Ctors(Vec<(usize, bool)>),
    /// Integers or chars: the domain segments and the largest key.
    Ints(Vec<(Bound, Bound)>, u128),
    /// Slices (`None`) or arrays of a known length.
    Slice(Option<usize>),
    /// Values lust does not list (floats, `str`, generics): only a
    /// wildcard covers them.
    Open,
}

/// What a row's first cell matches.
enum Head<'p> {
    Wild,
    /// Matches nothing the check can name: the row never covers a
    /// constructor and is not a wildcard either.
    Opaque,
    /// Constructor `i` with its field cells.
    Ctor(u32, Vec<Cell<'p>>),
    Range(Bound, Bound),
    /// A slice pattern: prefix, suffix, and whether a `..` middle sits
    /// between them.
    Slice(Vec<Cell<'p>>, Vec<Cell<'p>>, bool),
}

/// One constructor from the split of a column, with the field count
/// the specialized rows get.
#[derive(Clone, Copy)]
enum Split {
    Ctor(u32, usize),
    Range(Bound, Bound),
    /// Slices of exactly this length.
    Len(usize),
    /// Slices of this length or longer, checked at this length.
    MinLen(usize),
}

/// Specialization steps one exhaustiveness check may take. A check
/// that runs out answers "not covered": a reject, never a wrong accept.
const FUEL: u32 = 200_000;

impl<'p> Ctx<'p> {
    /// True when `pats`, all matching values of one type, together
    /// match every value (rustc's usefulness of a wildcard row).
    /// For patterns matched on a place behind a dereference
    /// (`match *r`, `let Ok(x) = *r`), empty constructors do
    /// not count there, as in rustc.
    pub(super) fn covers_behind(&self, pats: &[PatId], behind: bool) -> bool {
        let rows: Vec<Vec<Cell<'p>>> = pats.iter().map(|&p| vec![Cell::Pat(p)]).collect();
        let fuel = std::cell::Cell::new(FUEL);
        self.rows_cover_all(rows, vec![behind], &fuel)
    }

    /// True when this pattern's place sits behind a reference:
    /// an explicit `&` pattern, or the auto-derefs match
    /// ergonomics recorded for it.
    fn pat_behind_ref(&self, pat: PatId) -> bool {
        let mut pat = pat;
        loop {
            match self.p.ast.pat(pat) {
                Pat::At { inner, .. } | Pat::Paren(inner) => pat = *inner,
                Pat::Ref { .. } => return true,
                _ => return self.out.pat_derefs[(pat.0 - self.out.pat_base) as usize] > 0,
            }
        }
    }

    /// True when the rows together match every tuple of values for
    /// their columns. `behind[i]` says column `i`'s place sits
    /// behind a dereference, where empty constructors do not
    /// count.
    fn rows_cover_all(&self, rows: Vec<Vec<Cell<'p>>>, behind: Vec<bool>, fuel: &std::cell::Cell<u32>) -> bool {
        if fuel.get() == 0 {
            return false;
        }
        fuel.set(fuel.get() - 1);
        if rows.is_empty() {
            return false;
        }
        if rows[0].is_empty() {
            return true;
        }
        let mut flat: Vec<Vec<Cell<'p>>> = Vec::with_capacity(rows.len());
        for row in rows {
            self.expand_or_head(row, &mut flat);
        }
        let rows = flat;
        let head_cell = rows.iter().map(|r| r[0]).find(|&c| !self.is_wild(c));
        let ty = head_cell.map(|c| match c {
            Cell::Pat(p) => self.pat_ty_of(self.pat_unwrap(p)),
            Cell::Val(_, t) => t,
            Cell::Wild => ERROR,
        });
        // A const value column counts as behind a reference: empty
        // constructors then need rows, which can only reject more.
        let behind_col = behind.first().copied().unwrap_or(false)
            || head_cell.is_some_and(|c| match c {
                Cell::Pat(p) => self.pat_behind_ref(p),
                _ => true,
            });
        let splits = match ty {
            None => None,
            Some(t) => {
                let col = self.column(t, behind_col);
                let heads: Vec<Head<'p>> = rows.iter().map(|r| self.cell_head(r[0], &col)).collect();
                self.split(&col, &heads).map(|s| (s, heads))
            }
        };
        let Some((splits, heads)) = splits else {
            // Some constructor is in no head (or the column is all
            // wildcards): the rows cover it only through their wildcard
            // heads, and then cover everything only if those rows do.
            let default: Vec<Vec<Cell<'p>>> = rows.iter().filter(|r| self.is_wild(r[0])).map(|r| r[1..].to_vec()).collect();
            return self.rows_cover_all(default, behind[1.min(behind.len())..].to_vec(), fuel);
        };
        splits.into_iter().all(|s| {
            let spec: Vec<Vec<Cell<'p>>> = rows.iter().zip(&heads).filter_map(|(r, h)| self.specialize(r, h, s)).collect();
            let n = match s {
                Split::Ctor(_, n) | Split::Len(n) | Split::MinLen(n) => n,
                Split::Range(..) => 0,
            };
            let mut spec_behind = vec![behind_col; n];
            spec_behind.extend_from_slice(&behind[1.min(behind.len())..]);
            self.rows_cover_all(spec, spec_behind, fuel)
        })
    }

    /// Pushes `row`, or one row per alternative when its head is an
    /// or-pattern (nested alternatives too).
    fn expand_or_head(&self, row: Vec<Cell<'p>>, out: &mut Vec<Vec<Cell<'p>>>) {
        if let Cell::Pat(p) = row[0]
            && let Pat::Or(alts) = self.p.ast.pat(self.pat_unwrap(p))
        {
            for &a in alts {
                let mut r = row.clone();
                r[0] = Cell::Pat(a);
                self.expand_or_head(r, out);
            }
        } else {
            out.push(row);
        }
    }

    fn pat_ty_of(&self, pat: PatId) -> Ty {
        self.out.pat_ty[(pat.0 - self.out.pat_base) as usize]
    }

    /// True when the cell matches any value (see `cell_is_wild`).
    fn is_wild(&self, c: Cell<'p>) -> bool {
        match c {
            Cell::Wild => true,
            Cell::Pat(p) => self.cell_is_wild(p),
            Cell::Val(..) => false,
        }
    }

    /// Reads a cell as a head for a column of kind `col`.
    fn cell_head(&self, c: Cell<'p>, col: &Col) -> Head<'p> {
        match c {
            Cell::Wild => Head::Wild,
            Cell::Pat(p) => self.head(p, col),
            Cell::Val(v, t) => self.val_head(v, t, col),
        }
    }

    /// True when the pattern matches any value: `_`, a plain binding,
    /// or a `..` (after `@`, parentheses, and `&`).
    fn cell_is_wild(&self, pat: PatId) -> bool {
        let pat = self.pat_unwrap(pat);
        match self.p.ast.pat(pat) {
            Pat::Wild | Pat::Rest => true,
            Pat::Binding { .. } => {
                let pi = (pat.0 - self.out.pat_base) as usize;
                self.out.pat_const[pi].is_none() && matches!(self.out.pat_res[pi], PatRes::Binding { .. } | PatRes::None)
            }
            _ => false,
        }
    }

    /// True when no value of type `t` exists, as far as
    /// exhaustiveness can tell: `!`, an enum whose every variant
    /// has an uninhabited field (or none at all), a struct,
    /// tuple, or non-empty array with an uninhabited part, and
    /// `Result` with both sides uninhabited. References, raw
    /// pointers, and foreign types such as `Box<_>` are never
    /// uninhabited here, as in rustc. `args` substitutes the
    /// type parameters of the Adt a field type came from.
    pub(super) fn ty_uninhabited(&self, t: Ty, args: &[GenericArg], depth: u32) -> bool {
        if depth > 32 {
            return false;
        }
        match self.kind(t) {
            TyKind::Never => true,
            TyKind::Param(p) => match args.get(p.0 as usize).and_then(|g| g.as_ty()) {
                Some(inner) => self.ty_uninhabited(inner, &[], depth + 1),
                None => false,
            },
            TyKind::Tuple(ts) => self.tys.tys_of(ts).iter().any(|&e| self.ty_uninhabited(e, args, depth + 1)),
            TyKind::Array(elem, c) => match self.tys.const_kind(c) {
                ConstKind::Int(n) => n > 0 && self.ty_uninhabited(elem, args, depth + 1),
                _ => false,
            },
            TyKind::Adt(..) => {
                if self.tys.option_elem(t).is_some() {
                    return false;
                }
                if let Some((ok, err)) = self.tys.result_args(t) {
                    return self.ty_uninhabited(ok, args, depth + 1) && self.ty_uninhabited(err, args, depth + 1);
                }
                let Some(a) = self.tys.local_adt(t) else { return false };
                let own: Vec<GenericArg> = match self.tys.adt_parts(t) {
                    Some((_, aa)) => self.tys.args(aa).to_vec(),
                    None => Vec::new(),
                };
                let adt = self.p.adt(a);
                if adt.is_enum {
                    adt.variants.iter().all(|v| v.fields.iter().any(|f| self.ty_uninhabited(f.ty, &own, depth + 1)))
                } else {
                    adt.variants[0].fields.iter().any(|f| self.ty_uninhabited(f.ty, &own, depth + 1))
                }
            }
            _ => false,
        }
    }

    /// The constructors of type `t`, seen through references.
    fn column(&self, t: Ty, behind: bool) -> Col {
        let mut t = self.resolve(t);
        let mut behind = behind;
        while let TyKind::Ref(_, _, inner) = self.kind(t) {
            behind = true;
            t = self.resolve(inner);
        }
        match self.kind(t) {
            TyKind::Bool => Col::Ctors(vec![(0, false), (0, false)]),
            TyKind::Unit => Col::Ctors(vec![(0, false)]),
            TyKind::Tuple(ts) => {
                let elems: Vec<Ty> = self.tys.tys_of(ts).to_vec();
                let empty = !behind && elems.iter().any(|&e| self.ty_uninhabited(e, &[], 0));
                Col::Ctors(vec![(elems.len(), empty)])
            }
            TyKind::Int(it) => {
                let max = if it.bits() >= 128 { u128::MAX } else { (1u128 << it.bits()) - 1 };
                let dom = match it {
                    IntTy::Usize => ((1, 0), (3, 0)),
                    IntTy::Isize => ((0, 0), (3, 0)),
                    _ => ((1, 0), (2, 0)),
                };
                Col::Ints(vec![dom], max)
            }
            // The char domain skips the surrogates.
            TyKind::Char => Col::Ints(vec![((1, 0), (1, 0xD800)), ((1, 0xE000), (2, 0))], 0x10FFFF),
            TyKind::Slice(_) => Col::Slice(None),
            TyKind::Array(_, c) => match self.tys.const_kind(c) {
                ConstKind::Int(n) => Col::Slice(Some(n as usize)),
                _ => Col::Open,
            },
            TyKind::Adt(..) => {
                if let Some(elem) = self.tys.option_elem(t) {
                    Col::Ctors(vec![(0, false), (1, !behind && self.ty_uninhabited(elem, &[], 0))])
                } else if let Some((ok, err)) = self.tys.result_args(t) {
                    Col::Ctors(vec![(1, !behind && self.ty_uninhabited(ok, &[], 0)), (1, !behind && self.ty_uninhabited(err, &[], 0))])
                } else if matches!(crate::ty::hash::hash_parts(&self.tys, t), Some((crate::ty::hash::HashTy::Entry, _))) {
                    // A map Entry: `Occupied(_)` and `Vacant(_)`.
                    Col::Ctors(vec![(1, false), (1, false)])
                } else if let Some(a) = self.tys.local_adt(t) {
                    // A `#[non_exhaustive]` enum needs a wildcard
                    // outside its defining crate (rustc E0004); std's
                    // `atomic::Ordering` is one such enum.
                    if self.p.adt(a).is_enum
                        && self.p.adt(a).non_exhaustive
                        && crate::check::collect::adt_is_foreign(self.p, a, self.module)
                    {
                        return Col::Open;
                    }
                    let args: Vec<GenericArg> = match self.tys.adt_parts(t) {
                        Some((_, aa)) => self.tys.args(aa).to_vec(),
                        None => Vec::new(),
                    };
                    Col::Ctors(
                        self.p
                            .adt(a)
                            .variants
                            .iter()
                            .map(|v| (v.fields.len(), !behind && v.fields.iter().any(|f| self.ty_uninhabited(f.ty, &args, 0))))
                            .collect(),
                    )
                } else {
                    Col::Open
                }
            }
            _ => Col::Open,
        }
    }

    /// The value a const pattern compares against: `Some(Some(v))` for
    /// a bool, integer, or char const (a bool as 0 or 1), `Some(None)`
    /// for another const, `None` when the pattern is not a const.
    fn const_pat_value(&self, pat: PatId) -> Option<Option<i128>> {
        use crate::ty::consteval::ConstValue as CV;
        let pi = (pat.0 - self.out.pat_base) as usize;
        let cv = match self.out.pat_res[pi] {
            PatRes::PrimConst(v) => return Some(Some(v)),
            PatRes::ConstItem(idx) => Some(&self.p.consts[idx as usize].value),
            PatRes::BodyConst(ci) => Some(&self.p.body_consts[ci as usize].value),
            _ => self.out.pat_const[pi].as_ref(),
        }?;
        Some(match cv {
            CV::Int(v) => Some(*v),
            CV::Char(c) => Some(*c as i128),
            CV::Bool(b) => Some(*b as i128),
            _ => None,
        })
    }

    /// Reads a cell pattern as a head for a column of kind `col`.
    fn head(&self, pat: PatId, col: &Col) -> Head<'p> {
        let p = self.p;
        if self.cell_is_wild(pat) {
            return Head::Wild;
        }
        let pat = self.pat_unwrap(pat);
        let pi = (pat.0 - self.out.pat_base) as usize;
        // A scalar point (bool, integer, or char) as a head.
        let point = |v: Option<i128>| -> Head {
            match (col, v) {
                (Col::Ctors(c), Some(v @ (0 | 1))) if c.len() == 2 && c[0].0 == 0 && c[1].0 == 0 => Head::Ctor(v as u32, Vec::new()),
                (Col::Ints(_, max), Some(v)) => match self.int_key(v, pat) {
                    Some(k) => Head::Range((1, k), next_bound((1, k), *max)),
                    None => Head::Opaque,
                },
                _ => Head::Opaque,
            }
        };
        if let Some(v) = self.structural_const_value(pi) {
            return self.val_head(v, self.pat_ty_of(pat), col);
        }
        if let Some(v) = self.const_pat_value(pat) {
            return point(v);
        }
        // The field cells of a constructor with `n` fields, from tuple
        // elements that may hold one `..`.
        let positional = |elems: &[PatId], n: usize| -> Option<Vec<Cell<'p>>> {
            match elems.iter().position(|&e| matches!(p.ast.pat(e), Pat::Rest)) {
                None => (elems.len() == n).then(|| elems.iter().map(|&e| Cell::Pat(e)).collect()),
                Some(r) => {
                    let after = elems.len() - r - 1;
                    if r + after > n {
                        return None;
                    }
                    let mut cells: Vec<Cell<'p>> = elems[..r].iter().map(|&e| Cell::Pat(e)).collect();
                    cells.resize(n - after, Cell::Wild);
                    cells.extend(elems[r + 1..].iter().map(|&e| Cell::Pat(e)));
                    Some(cells)
                }
            }
        };
        let arity = |i: u32| -> Option<usize> {
            match col {
                Col::Ctors(c) => c.get(i as usize).map(|&(n, _)| n),
                _ => None,
            }
        };
        let ctor = |i: u32, cells: Option<Vec<Cell<'p>>>| -> Head<'p> {
            match (arity(i), cells) {
                (Some(n), Some(cells)) if cells.len() == n => Head::Ctor(i, cells),
                _ => Head::Opaque,
            }
        };
        // Option and Result constructors carry their own resolutions.
        let index = match self.out.pat_res[pi] {
            PatRes::Opt(some) => Some(some as u32),
            PatRes::Res(ok) => Some(!ok as u32),
            PatRes::Entry(occupied) => Some(!occupied as u32),
            PatRes::Variant(_, v) => Some(v),
            _ => None,
        };
        match p.ast.pat(pat) {
            Pat::Lit(l) => match p.ast.expr(*l) {
                Expr::Bool(b) => point(Some(*b as i128)),
                _ => match col {
                    Col::Ints(..) => point(self.const_scalar(*l)),
                    _ => Head::Opaque,
                },
            },
            Pat::Range { lo, hi, inclusive } => {
                let Col::Ints(dom, max) = col else { return Head::Opaque };
                let lo_b = match lo {
                    Some(e) => match self.const_scalar(*e).and_then(|v| self.int_key(v, pat)) {
                        Some(k) => (1, k),
                        None => return Head::Opaque,
                    },
                    None => dom[0].0,
                };
                let hi_b = match hi {
                    Some(e) => match self.const_scalar(*e).and_then(|v| self.int_key(v, pat)) {
                        Some(k) if *inclusive => next_bound((1, k), *max),
                        Some(k) => (1, k),
                        None => return Head::Opaque,
                    },
                    None => dom[dom.len() - 1].1,
                };
                if lo_b < hi_b { Head::Range(lo_b, hi_b) } else { Head::Opaque }
            }
            Pat::Tuple(elems) => ctor(0, arity(0).and_then(|n| positional(elems, n))),
            Pat::TupleStruct { elems, .. } => match index {
                Some(i) => ctor(i, arity(i).and_then(|n| positional(elems, n))),
                None => Head::Opaque,
            },
            Pat::Path(_) | Pat::QPath { .. } | Pat::Binding { .. } => match index {
                Some(i) => ctor(i, Some(Vec::new())),
                None => Head::Opaque,
            },
            Pat::Struct { fields, .. } => match (self.out.pat_res[pi], index) {
                (PatRes::Variant(a, v), Some(i)) => {
                    let defs = &p.adt(a).variants[v as usize].fields;
                    let mut cells: Vec<Cell<'p>> = vec![Cell::Wild; defs.len()];
                    for &(fname, sub) in fields {
                        match defs.iter().position(|f| f.name == fname) {
                            Some(j) => cells[j] = Cell::Pat(sub),
                            None => return Head::Opaque,
                        }
                    }
                    ctor(i, Some(cells))
                }
                _ => Head::Opaque,
            },
            Pat::Slice { prefix, middle, suffix } => match col {
                Col::Slice(_) => Head::Slice(
                    prefix.iter().map(|&x| Cell::Pat(x)).collect(),
                    suffix.iter().map(|&x| Cell::Pat(x)).collect(),
                    middle.is_some(),
                ),
                _ => Head::Opaque,
            },
            _ => Head::Opaque,
        }
    }

    /// The value of a structural const pattern (a const item whose
    /// value is not a scalar), read as patterns by `val_head`.
    fn structural_const_value(&self, pi: usize) -> Option<&'p crate::ty::consteval::ConstValue> {
        use crate::ty::consteval::ConstValue as CV;
        let p = self.p;
        let v = match self.out.pat_res[pi] {
            PatRes::ConstItem(idx) => &p.consts[idx as usize].value,
            PatRes::BodyConst(ci) => &p.body_consts[ci as usize].value,
            _ => return None,
        };
        (!matches!(v, CV::Int(_) | CV::Char(_) | CV::Bool(_) | CV::Float(..) | CV::Str(_))).then_some(v)
    }

    /// Reads a part of a structural const pattern's value at type `t`
    /// as the pattern rustc's const-to-pattern makes of it: a scalar
    /// point, a constructor with value cells, or array and slice
    /// elements, through references. Floats, `&str`, byte data, and
    /// anything else are opaque (rustc compares floats and strings as
    /// opaque values too; byte data only rejects more here).
    fn val_head(&self, v: &'p crate::ty::consteval::ConstValue, t: Ty, col: &Col) -> Head<'p> {
        use crate::ty::consteval::ConstValue as CV;
        let p = self.p;
        let mut t = self.resolve(t);
        let mut v = v;
        while let TyKind::Ref(_, _, inner) = self.kind(t) {
            t = self.resolve(inner);
            // A slice reference holds its backing array in place.
            if let CV::Ref(x) = v {
                v = x;
            }
        }
        let ctor = |i: u32, cells: Vec<Cell<'p>>| -> Head<'p> {
            match col {
                Col::Ctors(c) if c.get(i as usize).is_some_and(|&(n, _)| n == cells.len()) => Head::Ctor(i, cells),
                _ => Head::Opaque,
            }
        };
        let point = |x: i128| -> Head<'p> {
            match col {
                Col::Ints(_, max) => match self.scalar_key(x, t) {
                    Some(k) => Head::Range((1, k), next_bound((1, k), *max)),
                    None => Head::Opaque,
                },
                _ => Head::Opaque,
            }
        };
        match (self.kind(t), v) {
            (TyKind::Int(_), CV::Int(x)) => point(*x),
            (TyKind::Char, CV::Char(c)) => point(*c as i128),
            (TyKind::Bool, CV::Bool(b)) => ctor(*b as u32, Vec::new()),
            (TyKind::Unit, _) => ctor(0, Vec::new()),
            (TyKind::Tuple(ts), CV::Aggregate(vals)) => {
                let ts = self.tys.tys_of(ts);
                if ts.len() != vals.len() {
                    return Head::Opaque;
                }
                ctor(0, vals.iter().zip(ts).map(|(x, &ft)| Cell::Val(x, ft)).collect())
            }
            (TyKind::Array(el, _) | TyKind::Slice(el), CV::Aggregate(vals)) if matches!(col, Col::Slice(_)) => {
                Head::Slice(vals.iter().map(|x| Cell::Val(x, el)).collect(), Vec::new(), false)
            }
            (TyKind::Array(el, _) | TyKind::Slice(el), CV::Repeat(x, n)) if matches!(col, Col::Slice(_)) && *n <= 4096 => {
                Head::Slice(vec![Cell::Val(&**x, el); *n as usize], Vec::new(), false)
            }
            (_, CV::Variant(i, vals)) if self.tys.option_elem(t).is_some() => {
                let el = self.tys.option_elem(t).unwrap_or(ERROR);
                ctor(*i, vals.iter().map(|x| Cell::Val(x, el)).collect())
            }
            (_, CV::Variant(i, vals)) if self.tys.result_args(t).is_some() => {
                let (ok, err) = self.tys.result_args(t).unwrap_or((ERROR, ERROR));
                let ft = if *i == 0 { ok } else { err };
                ctor(*i, vals.iter().map(|x| Cell::Val(x, ft)).collect())
            }
            (_, CV::Aggregate(vals) | CV::Variant(_, vals)) => {
                let Some(a) = self.tys.local_adt(t) else { return Head::Opaque };
                let i = match v {
                    CV::Variant(i, _) if p.adt(a).is_enum => *i,
                    _ if !p.adt(a).is_enum => 0,
                    _ => return Head::Opaque,
                };
                let Some(variant) = p.adt(a).variants.get(i as usize) else { return Head::Opaque };
                if variant.fields.len() != vals.len() {
                    return Head::Opaque;
                }
                // A template Adt's field types may name its params:
                // such a cell's column is open, so it only rejects.
                ctor(i, vals.iter().zip(&variant.fields).map(|(x, f)| Cell::Val(x, f.ty)).collect())
            }
            _ => Head::Opaque,
        }
    }

    /// The order key of value `v` at the type of the scalar pattern
    /// `pat` (seen through references).
    fn int_key(&self, v: i128, pat: PatId) -> Option<u128> {
        let mut t = self.resolve(self.pat_ty_of(pat));
        while let TyKind::Ref(_, _, inner) = self.kind(t) {
            t = self.resolve(inner);
        }
        self.scalar_key(v, t)
    }

    /// The constructors to check for a column, split so that each one
    /// is either inside or outside every head. `None` when some
    /// constructor of the type is in no head.
    fn split(&self, col: &Col, heads: &[Head<'p>]) -> Option<Vec<Split>> {
        match col {
            Col::Open => None,
            Col::Ctors(ctors) => {
                // An empty constructor matches no value, so it
                // needs no row and no split (rustc's
                // `min_exhaustive_patterns`).
                let mut out = Vec::new();
                for (i, &(n, empty)) in ctors.iter().enumerate() {
                    if empty {
                        continue;
                    }
                    let i = i as u32;
                    if !heads.iter().any(|h| matches!(h, Head::Ctor(j, _) if *j == i)) {
                        return None;
                    }
                    out.push(Split::Ctor(i, n));
                }
                Some(out)
            }
            Col::Ints(dom, _) => {
                let mut cuts: Vec<Bound> = dom.iter().flat_map(|&(a, b)| [a, b]).collect();
                for h in heads {
                    if let Head::Range(a, b) = h {
                        cuts.extend([*a, *b]);
                    }
                }
                cuts.sort_unstable();
                cuts.dedup();
                let mut out = Vec::new();
                for w in cuts.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if !dom.iter().any(|&(lo, hi)| lo <= a && b <= hi) {
                        continue;
                    }
                    if !heads.iter().any(|h| matches!(*h, Head::Range(lo, hi) if lo <= a && b <= hi)) {
                        return None;
                    }
                    out.push(Split::Range(a, b));
                }
                Some(out)
            }
            Col::Slice(Some(n)) => {
                let s = Split::Len(*n);
                heads.iter().any(|h| slice_head_fits(h, s)).then_some(vec![s])
            }
            Col::Slice(None) => {
                // rustc's slice split: lengths below `l` one by one, and
                // every length from `l` up checked at `l`, where `l` is
                // past every fixed length and fits every `..` pattern.
                let (mut max_fixed, mut max_pre, mut max_suf) = (None::<usize>, 0, 0);
                for h in heads {
                    match h {
                        Head::Slice(a, b, false) => max_fixed = max_fixed.max(Some(a.len() + b.len())),
                        Head::Slice(a, b, true) => {
                            max_pre = max_pre.max(a.len());
                            max_suf = max_suf.max(b.len());
                        }
                        _ => {}
                    }
                }
                let l = max_fixed.map_or(0, |m| m + 1).max(max_pre + max_suf);
                let splits: Vec<Split> = (0..l).map(Split::Len).chain([Split::MinLen(l)]).collect();
                splits.iter().all(|&s| heads.iter().any(|h| slice_head_fits(h, s))).then_some(splits)
            }
        }
    }

    /// The row specialized to constructor `s`: the head's field cells
    /// then the rest of the row, or `None` when the head does not
    /// match `s`.
    fn specialize(&self, row: &[Cell<'p>], head: &Head<'p>, s: Split) -> Option<Vec<Cell<'p>>> {
        let mut out: Vec<Cell<'p>> = match (head, s) {
            (Head::Wild, Split::Ctor(_, n) | Split::Len(n) | Split::MinLen(n)) => vec![Cell::Wild; n],
            (Head::Wild, Split::Range(..)) => Vec::new(),
            (Head::Ctor(i, cells), Split::Ctor(j, _)) if *i == j => cells.clone(),
            (Head::Range(lo, hi), Split::Range(a, b)) if *lo <= a && b <= *hi => Vec::new(),
            (Head::Slice(pre, suf, _), Split::Len(n) | Split::MinLen(n)) if slice_head_fits(head, s) => {
                let mut cells: Vec<Cell<'p>> = pre.clone();
                cells.resize(n - suf.len(), Cell::Wild);
                cells.extend(suf.iter().copied());
                cells
            }
            _ => return None,
        };
        out.extend_from_slice(&row[1..]);
        Some(out)
    }
}

/// The bound just past `b` in a type whose largest key is `max`.
fn next_bound(b: Bound, max: u128) -> Bound {
    match b {
        (0, _) => (1, 0),
        (1, k) if k == max => (2, 0),
        (1, k) => (1, k + 1),
        _ => (3, 0),
    }
}

/// True when a slice head matches every slice of the split's lengths:
/// a fixed pattern only its own length, a `..` pattern every length
/// that fits its prefix and suffix.
fn slice_head_fits(h: &Head<'_>, s: Split) -> bool {
    match (h, s) {
        (Head::Slice(a, b, false), Split::Len(n)) => a.len() + b.len() == n,
        (Head::Slice(a, b, true), Split::Len(n) | Split::MinLen(n)) => a.len() + b.len() <= n,
        _ => false,
    }
}
