// The `str` split family in body checking (W-STD-B): `split`, `rsplit`,
// `split_terminator`, `rsplit_terminator`, `splitn`, `rsplitn`,
// `split_inclusive`, `split_whitespace`, `split_ascii_whitespace`,
// `lines`, `split_once`, and `rsplit_once`, over the patterns std's
// `Pattern` impls cover: `char`, `&str` (also `&&str` and `&String`),
// char arrays and slices, and `FnMut(char) -> bool` closures. The
// iterators are `IterKind::StrSplit` (ty/iter.rs); the runtime runs std's
// searchers and `SplitInternal` (runtime/core.c `lust_split_step`).
use super::*;
use crate::ty::iter::{self as it, SplitMode};

/// True when a split pattern type is a string needle (`&str`, `&&str`,
/// `&String`), whose searcher is not double-ended.
pub fn str_needle(tys: &impl TyIntern, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Ref(_, _, inner) => match tys.kind(inner) {
            TyKind::Str => true,
            TyKind::Ref(_, _, x) => tys.kind(x) == TyKind::Str,
            _ => tys.is_string_ty(inner),
        },
        _ => false,
    }
}

/// True for the pattern types lust runs (std's `Pattern` impls but the
/// fn-item predicates, which `is_pattern_fn_item` adds).
fn pattern_ok(tys: &impl TyIntern, t: Ty) -> bool {
    match tys.kind(t) {
        TyKind::Char | TyKind::Closure(..) => true,
        // A safe Rust-ABI `fn(char) -> bool` (ts-rust jsdoc.rs
        // `trim_part_end`): `FnMut(char) -> bool` like a closure.
        TyKind::FnPtr(sig) => {
            let s = tys.sig(sig).value;
            !s.is_unsafe && !s.extern_c && tys.tys_of(s.inputs) == [CHAR] && s.output == BOOL
        }
        TyKind::Array(e, _) => e == CHAR,
        TyKind::Ref(_, _, inner) => {
            str_needle(tys, t) || matches!(tys.kind(inner), TyKind::Array(e, _) | TyKind::Slice(e) if e == CHAR)
        }
        _ => false,
    }
}

impl<'p> Ctx<'p> {
    /// The split family on a `str` or `String` receiver; None for other
    /// methods and receivers.
    pub(super) fn str_split_method(&mut self, e: ExprId, base: Ty, derefs: u8, name: Sym, name_span: Span, args: &[ExprId]) -> Option<Ty> {
        let base = self.deep(base);
        if self.kind(base) != TyKind::Str && !self.tys.is_string_ty(base) {
            return None;
        }
        use SplitMode as M;
        // (mode, takes a count, takes a pattern, `Some(back)` for the
        // `_once` forms)
        let (mode, count, pat, once) = match self.p.interner.get(name) {
            "split" => (M::Split, false, true, None),
            "rsplit" => (M::RSplit, false, true, None),
            "split_terminator" => (M::SplitTerminator, false, true, None),
            "rsplit_terminator" => (M::RSplitTerminator, false, true, None),
            "splitn" => (M::SplitN, true, true, None),
            "rsplitn" => (M::RSplitN, true, true, None),
            "split_inclusive" => (M::SplitInclusive, false, true, None),
            "split_whitespace" => (M::SplitWhitespace, false, false, None),
            "split_ascii_whitespace" => (M::SplitAsciiWhitespace, false, false, None),
            "lines" => (M::Lines, false, false, None),
            "split_once" => (M::Split, false, true, Some(false)),
            "rsplit_once" => (M::Split, false, true, Some(true)),
            _ => return None,
        };
        let n = usize::from(count) + usize::from(pat);
        if args.len() != n {
            self.err("L0105", name_span, format!("this method takes {n} argument(s) but {} were given", args.len()), "Pass exactly the declared arguments.");
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        }
        if count {
            self.check_args(&[USIZE], &args[..1], name_span);
        }
        let p = match args.last().filter(|_| pat) {
            Some(&a) => {
                // A closure is checked against `FnMut(char) -> bool` (as the
                // adapters' closures are), so `|c| ..` needs no annotation;
                // a closure value must have that signature too.
                let closure_expr = matches!(self.p.ast.expr(a), Expr::Closure { .. });
                let t = if closure_expr {
                    let f = self.fn_mut_param(&[CHAR], BOOL, name_span);
                    self.check_args(&[f], &[a], name_span);
                    f
                } else {
                    self.expr(a, None)
                };
                let t = self.deep(t);
                if (matches!(self.kind(t), TyKind::Closure(..)) || self.is_pattern_fn_item(t)) && !closure_expr {
                    let f = self.fn_mut_param(&[CHAR], BOOL, name_span);
                    if !self.unify(f, t) {
                        self.mismatch(self.span(a), f, t);
                        return Some(ERROR);
                    }
                }
                if !pattern_ok(&self.tys, t) && !self.is_pattern_fn_item(t) {
                    if self.kind(t) != TyKind::Error {
                        let tn = self.tn(t);
                        self.err(
                            "L0612",
                            self.span(a),
                            format!("`{tn}` is not a str pattern lust takes"),
                            "Pass a char, a `&str`, a char array or slice, or a closure `|c: char| -> bool`.",
                        );
                    }
                    return Some(ERROR);
                }
                t
            }
            None => CHAR,
        };
        let b = match once {
            Some(false) => Builtin::StrSplitOnce,
            Some(true) => Builtin::StrRSplitOnce,
            None => Builtin::StrSplit,
        };
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
        Some(match once {
            Some(_) => {
                let ts = self.tys.intern_tys(&[REF_STR, REF_STR]);
                let pair = self.tys.intern(TyKind::Tuple(ts));
                self.tys.mk_option(pair)
            }
            None => it::mk_split(&mut self.tys, p, mode),
        })
    }

    /// `str::parse::<T>()` (tf-str): `Result<T, <T as FromStr>::Err>`
    /// over a `FromStr` obligation on `T` (the turbofish when
    /// written, else a fresh variable the use fixes), run by the
    /// `StrParse` builtin. None for other methods and receivers.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn str_parse_method(
        &mut self,
        e: ExprId,
        base: Ty,
        derefs: u8,
        name: Sym,
        name_span: Span,
        args: &[ExprId],
        turbofish: &[ast::GenericArg],
    ) -> Option<Ty> {
        if self.p.interner.get(name) != "parse" {
            return None;
        }
        let base = self.deep(base);
        if self.kind(base) != TyKind::Str && !self.tys.is_string_ty(base) {
            return None;
        }
        if !args.is_empty() {
            self.err("L0105", name_span, format!("this method takes 0 argument(s) but {} were given", args.len()), "Pass exactly the declared arguments.");
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        }
        let t = match turbofish.first() {
            Some(ast::GenericArg::Type(w)) => {
                if turbofish.len() > 1 {
                    self.err("L0105", name_span, "this method takes 1 type argument", "Write one target type.");
                }
                self.resolve_written(*w)
            }
            _ => self.new_var(false),
        };
        self.deferred_sized.push((t, name_span));
        let lang = self.p.gx.lang;
        let args_id = self.tys.intern_args(&[GenericArg::Ty(t)]);
        self.obl.push(obligations::Obligation::Trait {
            trait_ref: TraitRef { def: lang.from_str, args: args_id, bindings: EMPTY_BINDINGS },
            span: name_span,
            depth: 0,
        });
        // The error type is a fresh variable tied to the impl by an
        // associated-type equality, so it unifies as `T` settles
        // (a raw projection does not, in a `map_err` chain).
        let err = self.new_var(false);
        self.obl.push(obligations::Obligation::AssocEq {
            eq: AssocEqData { assoc: lang.assoc_from_str_err, trait_args: args_id, ty: err },
            span: name_span,
        });
        self.set_res(e, Res::BuiltinMethod { b: Builtin::StrParse, derefs, autoref: Autoref::Ref });
        Some(self.tys.mk_result(t, err))
    }

    /// True for a plain non-generic fn item used as a str pattern
    /// (`trim_start_matches(is_white_space_like)`): codegen calls it
    /// through an adapter (cgen/split.rs `split_pat`), as a closure body.
    fn is_pattern_fn_item(&self, t: Ty) -> bool {
        match self.kind(t) {
            TyKind::FnDef(def, args) => {
                self.p.is_fn_item_def(def) && self.tys.args(args).is_empty() && self.p.local_def(def.hash).is_some() && def != crate::ty::prim_fn_def()
            }
            _ => false,
        }
    }

    /// One checked pattern argument (the split family's rules): a
    /// closure literal is checked against `FnMut(char) -> bool`, a
    /// closure value must have that signature, and the type must be
    /// one lust takes. The pattern's type, or None after an error.
    fn check_pattern_arg(&mut self, a: ExprId, name_span: Span) -> Option<Ty> {
        let closure_expr = matches!(self.p.ast.expr(a), Expr::Closure { .. });
        let t = if closure_expr {
            let f = self.fn_mut_param(&[CHAR], BOOL, name_span);
            self.check_args(&[f], &[a], name_span);
            f
        } else {
            self.expr(a, None)
        };
        let t = self.deep(t);
        if (matches!(self.kind(t), TyKind::Closure(..)) || self.is_pattern_fn_item(t)) && !closure_expr {
            let f = self.fn_mut_param(&[CHAR], BOOL, name_span);
            if !self.unify(f, t) {
                self.mismatch(self.span(a), f, t);
                return None;
            }
        }
        if !pattern_ok(&self.tys, t) && !self.is_pattern_fn_item(t) {
            if self.kind(t) != TyKind::Error {
                let tn = self.tn(t);
                self.err(
                    "L0612",
                    self.span(a),
                    format!("`{tn}` is not a str pattern lust takes"),
                    "Pass a char, a `&str`, a char array or slice, or a closure `|c: char| -> bool`.",
                );
            }
            return None;
        }
        Some(t)
    }

    /// The str search and trim methods over a `Pattern` (tf-str):
    /// `find`, `rfind`, `contains`, `starts_with`, `ends_with`,
    /// `strip_prefix`, `strip_suffix`, the `trim_*_matches` family,
    /// the `matches` iterators, and `replace` and `replacen`, on a
    /// `str` or `String` receiver.
    /// The `char` and string-needle patterns of the search methods
    /// keep their older builtins (a char records `Coerce::CharPat`);
    /// char sets and closures run through `StrPat*` builtins over
    /// the split family's searchers. None for other methods.
    pub(super) fn str_search_method(&mut self, e: ExprId, base: Ty, derefs: u8, name: Sym, name_span: Span, args: &[ExprId]) -> Option<Ty> {
        let base = self.deep(base);
        if self.kind(base) != TyKind::Str && !self.tys.is_string_ty(base) {
            return None;
        }
        use Builtin as B;
        use it::MatchesMode as MM;
        #[derive(Clone, Copy, PartialEq)]
        enum Op {
            Find,
            Rfind,
            Contains,
            StartsWith,
            EndsWith,
            StripPrefix,
            StripSuffix,
            TrimMatches,
            TrimStartMatches,
            TrimEndMatches,
            Matches(MM),
            Replace,
            Replacen,
        }
        let op = match self.p.interner.get(name) {
            "replace" => Op::Replace,
            "replacen" => Op::Replacen,
            "find" => Op::Find,
            "rfind" => Op::Rfind,
            "contains" => Op::Contains,
            "starts_with" => Op::StartsWith,
            "ends_with" => Op::EndsWith,
            "strip_prefix" => Op::StripPrefix,
            "strip_suffix" => Op::StripSuffix,
            "trim_matches" | "trim_left_matches" | "trim_right_matches" => {
                // The deprecated aliases have the `trim_*` meanings:
                // left is start, right is end. `trim_matches` itself
                // trims both ends.
                match self.p.interner.get(name) {
                    "trim_left_matches" => Op::TrimStartMatches,
                    "trim_right_matches" => Op::TrimEndMatches,
                    _ => Op::TrimMatches,
                }
            }
            "trim_start_matches" => Op::TrimStartMatches,
            "trim_end_matches" => Op::TrimEndMatches,
            "matches" => Op::Matches(MM::Matches),
            "rmatches" => Op::Matches(MM::RMatches),
            "match_indices" => Op::Matches(MM::MatchIndices),
            "rmatch_indices" => Op::Matches(MM::RMatchIndices),
            _ => return None,
        };
        let n_args = match op {
            Op::Replace => 2,
            Op::Replacen => 3,
            _ => 1,
        };
        if args.len() != n_args {
            self.err(
                "L0105",
                name_span,
                format!("this method takes {n_args} argument(s) but {} were given", args.len()),
                "Pass exactly the declared arguments.",
            );
            for &a in args {
                self.expr(a, None);
            }
            return Some(ERROR);
        }
        let a = args[0];
        let Some(t) = self.check_pattern_arg(a, name_span) else { return Some(ERROR) };
        // `replace` and `replacen`: a char or string needle keeps the
        // older `&str` builtins; a char set or a predicate runs alloc's
        // `match_indices` loop over the split family's searchers.
        if matches!(op, Op::Replace | Op::Replacen) {
            let rest: &[Ty] = if op == Op::Replace { &[REF_STR] } else { &[REF_STR, USIZE] };
            self.check_args(rest, &args[1..], name_span);
            let b = if self.kind(t) == TyKind::Char || str_needle(&self.tys, t) {
                if self.kind(t) == TyKind::Char {
                    let ix = self.ei(a);
                    self.out.coerce[ix] = Coerce::CharPat;
                } else {
                    self.coerce_expr(a, t, REF_STR);
                }
                if op == Op::Replace { B::StrReplace } else { B::StrReplacen }
            } else if op == Op::Replace {
                B::StrPatReplace
            } else {
                B::StrPatReplacen
            };
            self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
            return Some(STRING);
        }
        self.reject_unsized_value(a);
        let needle = str_needle(&self.tys, t);
        // std's `trim_matches` needs a `DoubleEndedSearcher`, which
        // string needles do not have: rustc rejects `&str` there.
        if op == Op::TrimMatches && needle {
            self.err(
                "L0612",
                self.span(a),
                "`trim_matches` does not take a `&str` pattern".to_string(),
                "Pass a char, a char array or slice, or a closure `|c: char| -> bool`, or use `trim_start_matches` and `trim_end_matches`.",
            );
            return Some(ERROR);
        }
        let char_like = !needle && self.kind(t) != TyKind::Char;
        let (b, ret) = match op {
            Op::Matches(mode) => (B::StrMatches, it::mk_matches(&mut self.tys, t, mode)),
            Op::TrimMatches => (B::StrTrimMatches, REF_STR),
            Op::TrimStartMatches => (B::StrTrimStartMatches, REF_STR),
            Op::TrimEndMatches => (B::StrTrimEndMatches, REF_STR),
            Op::Find if char_like => (B::StrPatFind, self.tys.mk_option(USIZE)),
            Op::Rfind if char_like => (B::StrPatRfind, self.tys.mk_option(USIZE)),
            Op::Contains if char_like => (B::StrPatContains, BOOL),
            Op::StartsWith if char_like => (B::StrPatStartsWith, BOOL),
            Op::EndsWith if char_like => (B::StrPatEndsWith, BOOL),
            Op::StripPrefix if char_like => (B::StrPatStripPrefix, self.tys.mk_option(REF_STR)),
            Op::StripSuffix if char_like => (B::StrPatStripSuffix, self.tys.mk_option(REF_STR)),
            Op::Find => (B::StrFind, self.tys.mk_option(USIZE)),
            Op::Rfind => (B::StrRfind, self.tys.mk_option(USIZE)),
            Op::Contains => (B::StrContains, BOOL),
            Op::StartsWith => (B::StrStartsWith, BOOL),
            Op::EndsWith => (B::StrEndsWith, BOOL),
            Op::StripPrefix => (B::StrStripPrefix, self.tys.mk_option(REF_STR)),
            Op::StripSuffix => (B::StrStripSuffix, self.tys.mk_option(REF_STR)),
            Op::Replace | Op::Replacen => unreachable!("returned above"),
        };
        // The older builtins take the pattern as a `&str`: a char
        // becomes its encoding (`Coerce::CharPat`), and a needle
        // coerces as before.
        if matches!(b, B::StrFind | B::StrRfind | B::StrContains | B::StrStartsWith | B::StrEndsWith | B::StrStripPrefix | B::StrStripSuffix) {
            if self.kind(t) == TyKind::Char {
                let ix = self.ei(a);
                self.out.coerce[ix] = Coerce::CharPat;
            } else {
                self.coerce_expr(a, t, REF_STR);
            }
        }
        self.set_res(e, Res::BuiltinMethod { b, derefs, autoref: Autoref::Ref });
        Some(ret)
    }
}
