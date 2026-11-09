// The str split family in codegen (W-STD-B): one runtime step per
// `next`/`next_back` (runtime/core.c `lust_split_step`) and
// `split_once`/`rsplit_once`, with the pattern described by a `lust_pat`
// built from its type: a char, a string needle (`&str`, `&&str`,
// `&String`), a char set (an array, a reference to one, or a slice), or
// a closure, called through a small adapter (`void *env, uint32_t c`).
use super::*;
use crate::ty::iter::{self as it, SplitMode};

impl FnGen<'_, '_> {
    /// One split builtin as the runtime runs it, or None for every other
    /// builtin. `StrSplit` itself is expanded in lower/iter.rs.
    pub(super) fn split_builtin(&mut self, b: Builtin, ops: &[Operand], dest: &CTy, loc: &str, post: &mut Vec<String>) -> Option<String> {
        use Builtin as B;
        if !matches!(
            b,
            B::StrSplitNext
                | B::StrSplitNextBack
                | B::StrSplitUpper
                | B::StrSplitOnce
                | B::StrRSplitOnce
                | B::StrMatchesNext
                | B::StrMatchesNextBack
                | B::StrPatFind
                | B::StrPatRfind
                | B::StrPatContains
                | B::StrPatStartsWith
                | B::StrPatEndsWith
                | B::StrPatStripPrefix
                | B::StrPatStripSuffix
                | B::StrTrimMatches
                | B::StrTrimStartMatches
                | B::StrTrimEndMatches
                | B::StrPatReplace
                | B::StrPatReplacen
        ) {
            return None;
        }
        let mut vals = Vec::new();
        let mut tys = Vec::new();
        for o in ops {
            let (v, t) = self.operand(o, loc, post);
            vals.push(v);
            tys.push(t);
        }
        self.note_held(dest);
        let dc = ctype(dest);
        let t = self.tmp();
        match b {
            B::StrSplitNext | B::StrSplitNextBack => {
                let (p, mode) = self.split_parts(&ops[0]);
                let s = &vals[0];
                let pat = self.split_pat(p, mode, &format!("(&({s})->f7)"));
                let back = u8::from(b == B::StrSplitNextBack);
                let _ = writeln!(
                    self.out,
                    "    lust_str {t}_s; lust_pat {t}_p = {pat}; int {t}_k = lust_split_step((lust_split*)({s}), &{t}_p, {}, {back}, &{t}_s);",
                    mode as u8
                );
                let some = self.make_variant(dest, 1, &[format!("{t}_s")]);
                let none = self.make_variant(dest, 0, &[]);
                let _ = writeln!(self.out, "    {dc} {t} = {t}_k ? {some} : {none};");
                Some(t)
            }
            B::StrMatchesNext | B::StrMatchesNextBack => {
                let (p, mode) = self.matches_parts(&ops[0]);
                let s = &vals[0];
                let pat = self.split_pat(p, SplitMode::Split, &format!("(&({s})->f7)"));
                let back = u8::from((b == B::StrMatchesNextBack) != mode.reversed());
                let _ = writeln!(
                    self.out,
                    "    lust_str {t}_s; uint64_t {t}_i; lust_pat {t}_p = {pat}; int {t}_k = lust_matches_step((lust_split*)({s}), &{t}_p, {back}, &{t}_i, &{t}_s);"
                );
                let pair = CTy::Tuple(vec![CTy::Int(IntTy::Usize), CTy::Ref(Box::new(CTy::Str))]);
                self.note_held(&pair);
                let some = self.make_variant(dest, 1, &[format!("(({}){{ {t}_i, {t}_s }})", ctype(&pair))]);
                let none = self.make_variant(dest, 0, &[]);
                let _ = writeln!(self.out, "    {dc} {t} = {t}_k ? {some} : {none};");
                Some(t)
            }
            B::StrPatFind
            | B::StrPatRfind
            | B::StrPatContains
            | B::StrPatStartsWith
            | B::StrPatEndsWith
            | B::StrPatStripPrefix
            | B::StrPatStripSuffix
            | B::StrTrimMatches
            | B::StrTrimStartMatches
            | B::StrTrimEndMatches => {
                // (`&str` or `&String`, pattern by value), as
                // `split_once` above.
                let hay = str_of(&vals[0], &tys[0]);
                let p = self.op_lust_ty(&ops[1]);
                let _ = writeln!(self.out, "    {} {t}_v = {};", ctype(&tys[1]), vals[1]);
                let pat = self.split_pat(p, SplitMode::Split, &format!("(&{t}_v)"));
                let _ = writeln!(self.out, "    lust_str {t}_h = {hay}; lust_pat {t}_p = {pat};");
                let drop_pat = |this: &mut Self| {
                    if needs_drop_c(this.p, &tys[1]) {
                        this.want(Glue::Drop, &tys[1]);
                        let _ = writeln!(this.out, "    drop_{}(&{t}_v);", mangle(&tys[1]));
                    }
                };
                let r = match b {
                    B::StrTrimMatches | B::StrTrimStartMatches | B::StrTrimEndMatches => {
                        let expr = match b {
                            B::StrTrimStartMatches => format!("lust_pat_trim_start({t}_h, &{t}_p)"),
                            B::StrTrimEndMatches => format!("lust_pat_trim_end({t}_h, &{t}_p)"),
                            _ => format!("lust_pat_trim_end(lust_pat_trim_start({t}_h, &{t}_p), &{t}_p)"),
                        };
                        let _ = writeln!(self.out, "    {dc} {t} = {expr};");
                        drop_pat(self);
                        t.clone()
                    }
                    B::StrPatContains | B::StrPatStartsWith | B::StrPatEndsWith => {
                        let call = match b {
                            B::StrPatContains => format!("lust_pat_match({t}_h, &{t}_p, 0, &{t}_a, &{t}_b)"),
                            B::StrPatStartsWith => format!("lust_pat_prefix({t}_h, &{t}_p, &{t}_b)"),
                            _ => format!("lust_pat_suffix({t}_h, &{t}_p, &{t}_a)"),
                        };
                        let _ = writeln!(self.out, "    uint64_t {t}_a, {t}_b; int {t}_k = {call};");
                        drop_pat(self);
                        format!("((uint8_t){t}_k)")
                    }
                    B::StrPatFind | B::StrPatRfind => {
                        let back = u8::from(b == B::StrPatRfind);
                        let _ = writeln!(
                            self.out,
                            "    uint64_t {t}_a, {t}_b; int {t}_k = lust_pat_match({t}_h, &{t}_p, {back}, &{t}_a, &{t}_b);"
                        );
                        let some = self.make_variant(dest, 1, &[format!("{t}_a")]);
                        let none = self.make_variant(dest, 0, &[]);
                        let _ = writeln!(self.out, "    {dc} {t} = {t}_k ? {some} : {none};");
                        drop_pat(self);
                        t.clone()
                    }
                    _ => {
                        // strip_prefix and strip_suffix.
                        let _ = writeln!(self.out, "    uint64_t {t}_x;");
                        let (call, val) = if b == B::StrPatStripPrefix {
                            (format!("lust_pat_prefix({t}_h, &{t}_p, &{t}_x)"), format!("lust_split_sub({t}_h, {t}_x, {t}_h.len)"))
                        } else {
                            (format!("lust_pat_suffix({t}_h, &{t}_p, &{t}_x)"), format!("lust_split_sub({t}_h, 0, {t}_x)"))
                        };
                        let _ = writeln!(self.out, "    int {t}_k = {call};");
                        let some = self.make_variant(dest, 1, &[val]);
                        let none = self.make_variant(dest, 0, &[]);
                        let _ = writeln!(self.out, "    {dc} {t} = {t}_k ? {some} : {none};");
                        drop_pat(self);
                        t.clone()
                    }
                };
                Some(r)
            }
            B::StrPatReplace | B::StrPatReplacen => {
                // (`&str` or `&String`, pattern by value, `to`, count):
                // alloc's `match_indices` loop (runtime `lust_pat_replacen`).
                let hay = str_of(&vals[0], &tys[0]);
                let to = str_of(&vals[2], &tys[2]);
                let p = self.op_lust_ty(&ops[1]);
                let _ = writeln!(self.out, "    {} {t}_v = {};", ctype(&tys[1]), vals[1]);
                let pat = self.split_pat(p, SplitMode::Split, &format!("(&{t}_v)"));
                let (count, cap) = if b == B::StrPatReplace { ("0xffffffffffffffffull".to_string(), 0) } else { (vals[3].clone(), 32) };
                let _ = writeln!(self.out, "    lust_pat {t}_p = {pat}; {dc} {t} = lust_pat_replacen({hay}, &{t}_p, {to}, {count}, {cap});");
                if needs_drop_c(self.p, &tys[1]) {
                    self.want(Glue::Drop, &tys[1]);
                    let _ = writeln!(self.out, "    drop_{}(&{t}_v);", mangle(&tys[1]));
                }
                Some(t)
            }
            B::StrSplitUpper => Some(format!("lust_split_upper((const lust_split*)({}))", vals[0])),
            _ => {
                // (`&str` or `&String`, pattern by value).
                let hay = match &tys[0] {
                    CTy::Ref(x) if **x == CTy::String => format!("lust_string_as_str({})", vals[0]),
                    _ => vals[0].clone(),
                };
                let p = self.op_lust_ty(&ops[1]);
                let _ = writeln!(self.out, "    {} {t}_v = {};", ctype(&tys[1]), vals[1]);
                let pat = self.split_pat(p, SplitMode::Split, &format!("(&{t}_v)"));
                let back = u8::from(b == B::StrRSplitOnce);
                let _ = writeln!(
                    self.out,
                    "    lust_str {t}_a, {t}_b; lust_pat {t}_p = {pat}; int {t}_k = lust_str_split_once({hay}, &{t}_p, {back}, &{t}_a, &{t}_b);"
                );
                if needs_drop_c(self.p, &tys[1]) {
                    self.want(Glue::Drop, &tys[1]);
                    let _ = writeln!(self.out, "    drop_{}(&{t}_v);", mangle(&tys[1]));
                }
                let pair = CTy::Tuple(vec![CTy::Ref(Box::new(CTy::Str)), CTy::Ref(Box::new(CTy::Str))]);
                self.note_held(&pair);
                let some = self.make_variant(dest, 1, &[format!("(({}){{ {t}_a, {t}_b }})", ctype(&pair))]);
                let none = self.make_variant(dest, 0, &[]);
                let _ = writeln!(self.out, "    {dc} {t} = {t}_k ? {some} : {none};");
                Some(t)
            }
        }
    }

    /// The type of an operand: a constant's, or its place's (the local's
    /// type through its projections: a closure pattern can sit in an
    /// array element or a tuple field, `s.find(fs[0])`).
    fn op_lust_ty(&self, o: &Operand) -> Ty {
        match o {
            Operand::Move(_) | Operand::Copy(_) => self.callable_place_ty(o),
            Operand::Const(Const::Str(_)) => REF_STR,
            Operand::Const(Const::Char(_)) => CHAR,
            Operand::Const(_) => ERROR,
        }
    }

    /// The pattern type and mode of a `&mut StrMatches<P, M>` operand.
    fn matches_parts(&self, o: &Operand) -> (Ty, it::MatchesMode) {
        let mut t = self.op_lust_ty(o);
        while let TyKind::Ref(_, _, i) = self.body.tys.kind(t) {
            t = i;
        }
        match it::iter_parts(&self.body.tys, t) {
            Some((it::IterKind::StrMatches, a)) => (a[0], it::matches_mode(&self.body.tys, a[1]).unwrap_or(it::MatchesMode::Matches)),
            _ => (CHAR, it::MatchesMode::Matches),
        }
    }

    /// The pattern type and mode of a `&mut StrSplit<P, M>` operand.
    fn split_parts(&self, o: &Operand) -> (Ty, SplitMode) {
        let mut t = self.op_lust_ty(o);
        while let TyKind::Ref(_, _, i) = self.body.tys.kind(t) {
            t = i;
        }
        match it::iter_parts(&self.body.tys, t) {
            Some((it::IterKind::StrSplit, a)) => (a[0], it::split_mode(&self.body.tys, a[1]).unwrap_or(SplitMode::Split)),
            _ => (CHAR, SplitMode::Split),
        }
    }

    /// A `lust_pat` initializer for pattern type `p` stored at the C
    /// pointer `ptr` (the whitespace and line modes ignore it).
    fn split_pat(&mut self, p: Ty, mode: SplitMode, ptr: &str) -> String {
        let n_of = |this: &Self, c: ConstId| match this.body.tys.const_kind(c) {
            ConstKind::Int(n) => n as u64,
            _ => 0,
        };
        match mode {
            SplitMode::SplitWhitespace => return "((lust_pat){ .kind = LUST_PAT_SPACE })".into(),
            SplitMode::SplitAsciiWhitespace => return "((lust_pat){ .kind = LUST_PAT_ASCII_SPACE })".into(),
            SplitMode::Lines => return "((lust_pat){ .kind = LUST_PAT_CHAR, .ch = 10 })".into(),
            _ => {}
        }
        match self.body.tys.kind(p) {
            TyKind::Char => format!("((lust_pat){{ .kind = LUST_PAT_CHAR, .ch = *{ptr} }})"),
            TyKind::Array(_, c) => format!("((lust_pat){{ .kind = LUST_PAT_SET, .set = ({ptr})->v, .n = {} }})", n_of(self, c)),
            TyKind::Closure(def, _) => {
                let f = self.p.gx.closures[&def].body_fn;
                let a = self.split_adapter(f, true);
                format!("((lust_pat){{ .kind = LUST_PAT_PRED, .pred = {a}, .env = (void*){ptr} }})")
            }
            // A plain fn item (check/split.rs `is_pattern_fn_item`): no env.
            TyKind::FnDef(def, _) if let Some(f) = self.p.local_def(def.hash) => {
                let a = self.split_adapter(f, false);
                format!("((lust_pat){{ .kind = LUST_PAT_PRED, .pred = {a}, .env = (void*){ptr} }})")
            }
            // A `fn(char) -> bool` pointer: the env is the stored pointer.
            TyKind::FnPtr(_) => {
                let ct = cty(&self.body.tys, p);
                self.note_held(&ct);
                let name = format!("lust_splitp_{}_{}", super::names::fn_tag(self.slots.of(self.body.fn_id)), self.adapters);
                self.adapters += 1;
                let _ = writeln!(self.prelude, "static uint8_t {name}(void *env, uint32_t c) {{\n  return (*({}*)env)(c);\n}}", ctype(&ct));
                format!("((lust_pat){{ .kind = LUST_PAT_PRED, .pred = {name}, .env = (void*){ptr} }})")
            }
            TyKind::Ref(_, _, inner) => match self.body.tys.kind(inner) {
                TyKind::Str => format!("((lust_pat){{ .kind = LUST_PAT_STR, .needle = *{ptr} }})"),
                TyKind::Ref(..) => format!("((lust_pat){{ .kind = LUST_PAT_STR, .needle = **{ptr} }})"),
                TyKind::Array(_, c) => format!("((lust_pat){{ .kind = LUST_PAT_SET, .set = (*{ptr})->v, .n = {} }})", n_of(self, c)),
                TyKind::Slice(_) => format!("((lust_pat){{ .kind = LUST_PAT_SET, .set = (*{ptr}).ptr, .n = (*{ptr}).len }})"),
                _ => format!("((lust_pat){{ .kind = LUST_PAT_STR, .needle = lust_string_as_str(*{ptr}) }})"),
            },
            _ => "((lust_pat){ .kind = LUST_PAT_CHAR })".into(),
        }
    }

    /// The `uint8_t f(void *env, uint32_t c)` adapter that calls a
    /// predicate closure's body (its env is the stored closure), or,
    /// without `env`, a plain fn item that takes the char alone.
    fn split_adapter(&mut self, f: LocalDefId, env: bool) -> String {
        let name = format!("lust_splitp_{}_{}", super::names::fn_tag(self.slots.of(self.body.fn_id)), self.adapters);
        self.adapters += 1;
        let mut body = String::new();
        let (r, _) = if env {
            self.closure_call_c(&mut body, f, "env", &["c".to_string()])
        } else {
            self.fn_call_c(&mut body, f, &["c".to_string()])
        };
        let _ = writeln!(self.prelude, "static uint8_t {name}(void *env, uint32_t c) {{\n{body}  return {r};\n}}");
        name
    }
}
