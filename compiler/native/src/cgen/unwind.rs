// Codegen for panic=unwind (docs/ARCH.md section 7.2): explicit
// propagation. After every call and every statement that can unwind,
// a frame checks for a new exception on its thread (one global load
// when no thread has one) and jumps to its cleanup ladder; the
// ladder drops the live locals and its last block gives the exception
// back and returns. A statement in a cleanup block that ends with a
// new exception aborts. The runtime half is in runtime/panic.c.
use super::*;

/// What a statement or call does when it ends with a new exception.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Unw {
    /// Take the exception into the frame and run the cleanup from block `n`.
    Goto(u32),
    /// The statement is cleanup itself: a new exception aborts.
    Abort,
}

impl Unw {
    /// The C check that follows a call or statement.
    pub(super) fn check(self) -> String {
        match self {
            Unw::Goto(b) => format!("LUST_UNW({b});"),
            Unw::Abort => "LUST_UNW_ABORT();".into(),
        }
    }
}

impl FnGen<'_, '_> {
    /// `catch_unwind(f)` (lowered with no cleanup successor, lower/unwind.rs):
    /// calls `f`; under panic=unwind a panic in it comes back here and
    /// becomes `Err(payload)`, with `f`'s value dropped as `call_once`
    /// drops it. Under panic=abort a panic never returns, so it is `Ok`.
    pub(super) fn catch_unwind_call(&mut self, ops: &[Operand], loc: &str, post: &mut Vec<String>) -> String {
        let callee = self.callee_of(&ops[0]);
        let once = !matches!(callee, sync::Callee::Closure(_, false));
        let (env, env_ty) = self.operand(&ops[0], loc, post);
        self.note_held(&env_ty);
        let ret = self.callee_ret(&callee);
        let res = CTy::Result(Box::new(ret.clone()), Box::new(CTy::BoxDyn));
        self.note_held(&res);
        let ev = self.tmp();
        if self.p.unwind_tables {
            return self.catch_unwind_tables(&callee, &env, &env_ty, &ret, &res, once);
        }
        let _ = writeln!(self.out, "    {} {ev} = {env};", ctype(&env_ty));
        let mut call = String::new();
        let v = self.callee_call_c(&mut call, &callee, &ev, &[]);
        self.out.push_str(&call);
        let caught = self.tmp();
        if self.p.unwind {
            let _ = writeln!(self.out, "    lust_dyn {caught}p; int {caught} = lust_unw_pending && lust_unw_mine(); if ({caught}) {caught}p = lust_unw_catch();");
        }
        if !once && needs_drop_c(self.p, &env_ty) {
            self.want(Glue::Drop, &env_ty);
            let _ = writeln!(self.out, "    drop_{}(&{ev});", mangle(&env_ty));
        }
        let r = self.tmp();
        let ok = self.make_variant(&res, 0, &[v]);
        if self.p.unwind {
            let err = self.make_variant(&res, 1, &[format!("{caught}p")]);
            let _ = writeln!(self.out, "    {} {r} = {caught} ? {err} : {ok};", ctype(&res));
        } else {
            let _ = writeln!(self.out, "    {} {r} = {ok};", ctype(&res));
        }
        r
    }
}

impl FnGen<'_, '_> {
    /// `catch_unwind(f)` with unwind tables: the call of `f` goes in a thunk (`cu<tag>_<n>`,
    /// in the unit before this fn) that `lust_unw_try` runs under a catch point. The env and
    /// the result sit in one context struct, so a panic leaves this frame's locals alone.
    fn catch_unwind_tables(&mut self, callee: &sync::Callee, env: &str, env_ty: &CTy, ret: &CTy, res: &CTy, once: bool) -> String {
        let tag = names::fn_tag(self.slots.of(self.body.fn_id));
        let k = self.tmp();
        let name = format!("cu{tag}_{k}");
        let mut call = String::new();
        let v = self.callee_call_c(&mut call, callee, "(c_->ev)", &[]);
        let _ = writeln!(
            self.prelude,
            "struct {name} {{ {} ev; {} r; }};\nstatic void {name}_t(void *p_) {{\n  struct {name} *c_ = (struct {name} *)p_;\n{call}  c_->r = {v};\n}}",
            ctype(env_ty),
            ctype(ret)
        );
        let (c, caught) = (self.tmp(), self.tmp());
        let _ = writeln!(self.out, "    struct {name} {c};\n    {c}.ev = {env};");
        let _ = writeln!(
            self.out,
            "    lust_dyn {caught}p; int {caught} = lust_unw_try({name}_t, &{c}); if ({caught}) {caught}p = lust_unw_catch();"
        );
        if !once && needs_drop_c(self.p, env_ty) {
            self.want(Glue::Drop, env_ty);
            let _ = writeln!(self.out, "    drop_{}(&{c}.ev);", mangle(env_ty));
        }
        let r = self.tmp();
        let ok = self.make_variant(res, 0, &[format!("{c}.r")]);
        let err = self.make_variant(res, 1, &[format!("{caught}p")]);
        let _ = writeln!(self.out, "    {} {r} = {caught} ? {err} : {ok};", ctype(res));
        r
    }
}

/// Unwind tables (`Program::unwind_tables`, runtime/panic.c mode 2): per local, the C names
/// the fn's frame struct holds because the cleanup blocks' C (`cl_text`) reads the local,
/// its drop flag or one of its move flags; empty for every other local. The frame holds a
/// member local's drop flag and move flags, and a pointer to the local itself (which stays
/// a local of the fn), so the cleanup fn sees what the fn body last wrote. Promoted
/// constants (static storage) are never members.
pub(super) fn frame_members(cl_text: &str, n: usize, move_paths: &[Vec<Vec<Step>>], local_drop: &[bool], promoted: &[Local]) -> Vec<Vec<String>> {
    let mut used = vec![false; n];
    let b = cl_text.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_lit(b, i),
            c if is_ident(c) && (i == 0 || !is_ident(b[i - 1])) => {
                let start = i;
                while i < b.len() && is_ident(b[i]) {
                    i += 1;
                }
                let w = &cl_text[start..i];
                let num = |s: &str| s.parse::<usize>().ok().filter(|&k| k < n);
                let local = match w.as_bytes()[0] {
                    b'l' | b'd' => num(&w[1..]),
                    b'm' => w[1..].split_once('_').and_then(|(a, k)| num(a).filter(|_| k.parse::<u32>().is_ok())),
                    _ => None,
                };
                if let Some(l) = local {
                    used[l] = true;
                }
            }
            _ => i += 1,
        }
    }
    (0..n)
        .map(|l| {
            if !used[l] || promoted.contains(&(l as Local)) {
                return Vec::new();
            }
            let mut names = vec![format!("l{l}")];
            if local_drop[l] {
                names.push(format!("d{l}"));
            }
            names.extend((0..move_paths[l].len()).map(|k| format!("m{l}_{k}")));
            names
        })
        .collect()
}

impl FnGen<'_, '_> {
    /// Unwind tables: the frame struct type of the fn `tag` names (`names::fn_tag`: its
    /// slot, or its stable id in release C) with the ladder entry `st`, then a pointer to
    /// each member local (`pl<N>`: the local's own name may be a macro, `overaligned_local`),
    /// its drop flag and move flags; and its cleanup fn's prototype, both before the fn.
    /// With `inline` (`frame_inline`) the cleanup fn is always_inline: clang inlines it into
    /// the landing pad, so `&L` never leaves the fn and SROA keeps `st` and the flags in
    /// registers, with no store to memory for them.
    pub(super) fn frame_struct(&self, tag: &str, members: &[Vec<String>], inline: bool) -> String {
        let mut s = format!("struct fr{tag} {{\n  int32_t st;\n");
        for (l, names) in members.iter().enumerate() {
            for nm in names {
                if nm.starts_with('l') {
                    let _ = writeln!(s, "  {} *p{nm};", ctype(&self.local_cty[l]));
                } else {
                    let _ = writeln!(s, "  uint8_t {nm};");
                }
            }
        }
        // The cleanup variable names `clw<tag>`: every scope exit runs it, and a normal
        // return left `st = 0`, so the call to the cleanup fn folds away there; an unwind
        // left a ladder entry and runs the cleanup.
        let _ = writeln!(
            s,
            "}};\n{} void cl{tag}(struct fr{tag} *Lc);\nstatic inline __attribute__((always_inline)) void clw{tag}(struct fr{tag} *Lc) {{ if (__builtin_expect(Lc->st != 0, 0)) cl{tag}(Lc); }}",
            cleanup_linkage(inline)
        );
        s
    }
}

/// Unwind tables: whether a fn's cleanup fn is inlined into its landing pad
/// (`frame_struct`). Not when its flags times its unwind points (calls and statements that
/// may unwind) pass `LUST_RELEASE_FRAME_INLINE` (default 100000; 0: never): SROA puts a phi
/// per flag on every unwind edge and then runs for minutes, as the LLVM backend found
/// (`LUST_LL_FLAGMEM`). Such a fn keeps the cleanup fn out of line, and its frame in memory.
pub(super) fn frame_inline(flags: usize, unwind_points: usize) -> bool {
    static LIMIT: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    let limit = *LIMIT.get_or_init(|| std::env::var("LUST_RELEASE_FRAME_INLINE").ok().and_then(|v| v.parse().ok()).unwrap_or(100_000));
    (flags + 1) * unwind_points <= limit
}

fn cleanup_linkage(inline: bool) -> &'static str {
    if inline { "static inline __attribute__((always_inline))" } else { "static" }
}

/// Unwind tables: the cleanup fn of the fn `tag` names, after it. The unwinder calls it (the
/// cleanup variable `L`) at each scope exit: a normal return stored `st = 0`; an unwind
/// left the ladder entry of the call or statement that panicked, so it runs the cleanup
/// blocks from there and returns, and the unwinder goes on. `cl_text` is the cleanup
/// blocks' C, emitted with the members' names: a member local is read through its pointer.
pub(super) fn cleanup_fn(out: &mut String, tag: &str, members: &[Vec<String>], cl_order: &[usize], cl_text: &str, inline: bool) {
    let names: Vec<&String> = members.iter().flatten().collect();
    for nm in &names {
        let _ = writeln!(out, "#undef {nm}");
    }
    let _ = writeln!(out, "{} void cl{tag}(struct fr{tag} *Lc) {{", cleanup_linkage(inline));
    for nm in &names {
        if nm.starts_with('l') {
            let _ = writeln!(out, "#define {nm} (*Lc->p{nm})");
        } else {
            let _ = writeln!(out, "#define {nm} (Lc->{nm})");
        }
    }
    out.push_str("  lust_exc ex;\n  (void)ex;\n  switch (Lc->st) {\n  case 0: return;\n");
    for b in cl_order {
        let _ = writeln!(out, "  case {b}: goto bb{b};");
    }
    out.push_str("  default: return;\n  }\n");
    out.push_str(cl_text);
    out.push_str("}\n");
    for nm in &names {
        let _ = writeln!(out, "#undef {nm}");
    }
}

/// Unwind tables: whether dropping a value of type `t` can panic, so its drop glue needs
/// catch points: some part of it has a user `Drop` impl, or a dyn value whose drop is not
/// known here. Conservative: a kind not listed counts as "can". `seen` stops at a type
/// already on the walk (a recursive type adds nothing new there).
pub(super) fn drop_may_unwind(p: &Program, t: &CTy, seen: &mut HashSet<CTy>) -> bool {
    use CTy::*;
    match t {
        Unit | Bool | Char | Int(_) | Str | String | Float(_) | Ref(_) | Ptr(_) | FatPtr(_) | FnPtr(..) | Chars | Slice(_) | StrDrain
        | ExitCode | Weak(_) | WeakDyn(_) | Guard(..) | DynGuard(_) => false,
        Dyn | BoxDyn | RcDyn | DynCell => true,
        Vec(e) | Option(e) | Boxed(e) | BoxSlice(e) | Rc(e) | Array(e, _) | VecIter(e) | ArrayIter(e, _) | OptIter(_, e) | VecDrain(e)
        | RefCell(e) | Cell(e) => drop_may_unwind(p, e, seen),
        Result(a, b) => drop_may_unwind(p, a, seen) || drop_may_unwind(p, b, seen),
        Tuple(es) => es.iter().any(|e| drop_may_unwind(p, e, seen)),
        Hash(_, args) => args.iter().any(|e| drop_may_unwind(p, e, seen)),
        Adt(a) => {
            if p.gx.drop_methods.contains_key(a) {
                return true;
            }
            if !seen.insert(t.clone()) {
                return false;
            }
            let fields: std::vec::Vec<CTy> = p.adt(*a).variants.iter().flat_map(|v| v.fields.iter().map(|f| cty(&p.tys, f.ty))).collect();
            fields.iter().any(|f| drop_may_unwind(p, f, seen))
        }
        _ => true,
    }
}

/// One call statement of drop glue that can panic (`glue_drop_stmts`): the byte range of
/// the statement in the glue text (through its `;`) and its catch-point form.
struct DropStmt {
    start: usize,
    end: usize,
    dtry: String,
}

/// Drop glue's call statements that can panic: a drop glue call `drop_X(a);` and a call
/// through a cast fn pointer (a user Drop, a dyn drop entry) `((cast)f)(a);`, each at the
/// start of a statement; with `cases`, also right after an enum glue's `case N:` label.
fn glue_drop_stmts(text: &str, cases: bool) -> Vec<DropStmt> {
    let starts = |b: &[u8], at: usize| stmt_start(b, at) || (cases && after_case_label(b, at));
    let b = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_lit(b, i),
            b'(' if b.get(i + 1) == Some(&b'(') && starts(b, i) => {
                // `((cast)callee)(arg);`
                let Some(close) = match_paren(b, i) else {
                    i += 1;
                    continue;
                };
                let callee = &text[i + 1..close];
                let Some(close2) = b.get(close + 1).filter(|&&c| c == b'(').and_then(|_| match_paren(b, close + 1)) else {
                    i += 1;
                    continue;
                };
                if b.get(close2 + 1) != Some(&b';') {
                    i += 1;
                    continue;
                }
                let arg = &text[close + 2..close2];
                out.push(DropStmt { start: i, end: close2 + 2, dtry: format!("lust_unw_dtry((void (*)(void *))({callee}), (void *)({arg}), &ux);") });
                i = close2 + 2;
            }
            c if is_ident(c) && (i == 0 || !is_ident(b[i - 1])) => {
                let start = i;
                while i < b.len() && is_ident(b[i]) {
                    i += 1;
                }
                let name = &text[start..i];
                if !name.starts_with("drop_") || !starts(b, start) || b.get(i) != Some(&b'(') {
                    continue;
                }
                let Some(close) = match_paren(b, i) else { continue };
                if b.get(close + 1) != Some(&b';') {
                    continue;
                }
                let arg = &text[i + 1..close];
                out.push(DropStmt { start, end: close + 2, dtry: format!("lust_unw_dtry((void (*)(void *)){name}, (void *)({arg}), &ux);") });
                i = close + 2;
            }
            _ => i += 1,
        }
    }
    out
}

/// Unwind tables: drop glue's call statements that can panic (`glue_drop_stmts`) as catch
/// points: each becomes `lust_unw_dtry((void (*)(void *))f, (void *)(a), &ux);`. Returns
/// the text and whether it made any. The fallback of `frame_glue`.
pub(super) fn dtry_calls(text: &str) -> (String, bool) {
    let stmts = glue_drop_stmts(text, false);
    let mut out = String::with_capacity(text.len() + 64);
    let mut copied = 0;
    for d in &stmts {
        out.push_str(&text[copied..d.start]);
        out.push_str(&d.dtry);
        copied = d.end;
    }
    out.push_str(&text[copied..]);
    (out, !stmts.is_empty())
}

/// Unwind tables: drop glue that can panic as one frame, with no work on the normal path
/// beyond a state store per drop. Before its k-th drop statement (`glue_drop_stmts`) the
/// glue stores `st = k` in its frame `L` (and the index of the element loop it is in);
/// a panic there unwinds through the frame's cleanup variable, whose fn runs the rest of
/// the glue from just after that statement (a copy of the body entered by `goto`, the loop
/// index restored), as Rust's landing pad drops the other fields and elements; the unwind
/// then goes on. A panic in that rest aborts (runtime/panic.c, panic in a destructor
/// during cleanup). Returns the whole definition of `drop_<m>`, or None for a body this
/// does not cover (none of its statements can panic, or a hash table's local iterator),
/// which keeps `dtry_calls`.
pub(super) fn frame_glue(m: &str, ct: &str, body: &str) -> Option<String> {
    // Every drop is a state (enum arms too): a panic in one with no state of its own
    // would resume at the last state stored and drop parts twice.
    let stmts = glue_drop_stmts(body, true);
    if stmts.is_empty() || body.contains("while (") || body.contains("#error") {
        return None;
    }
    const FOR_I: &str = "for (uint64_t i = ";
    let has_loop = body.contains(FOR_I);
    // The body of a one-line element loop `for (uint64_t i = ..; ..; i++) STMT;`.
    let in_loop = |d: &DropStmt| {
        let line = &body[body[..d.start].rfind('\n').map_or(0, |n| n + 1)..d.start];
        line.trim_start().starts_with(FOR_I)
    };
    // One drop with nothing after it (an Option's or a one-field struct's glue): a panic
    // there leaves nothing to drop, so the frame's cleanup would do nothing. The plain
    // body keeps the glue inlinable (`fn_unit_parts` inlines only plain glue).
    if let [d] = stmts.as_slice()
        && !has_loop
        && nothing_after(&body[d.end..])
    {
        return Some(format!("void drop_{m}({ct} *p) {{\n{body}}}\n"));
    }
    let (mut fast, mut rest) = (String::with_capacity(body.len() + 64), String::with_capacity(body.len() + 64));
    let mut copied = 0;
    for (k, d) in stmts.iter().enumerate() {
        let (pre, stmt) = (&body[copied..d.start], &body[d.start..d.end]);
        fast.push_str(pre);
        rest.push_str(pre);
        let save_i = if in_loop(d) { " L.i = i;" } else { "" };
        let _ = write!(fast, "{{ L.st = {};{save_i} {stmt} }}", k + 1);
        let _ = write!(rest, "{{ {stmt} r{}:; }}", k + 1);
        copied = d.end;
    }
    fast.push_str(&body[copied..]);
    rest.push_str(&body[copied..]);
    // In the rest, the loop index is the fn's (restored from the frame), and the body's
    // own declarations (`uint64_t *h = ..;`: pure reads through `p`) go before the jump.
    let rest = rest.replace(FOR_I, "for (i = ");
    let (mut decls, mut tail) = (String::new(), String::new());
    for line in rest.lines() {
        let t = line.trim_start();
        let is_decl = line.starts_with("  ")
            && !line.starts_with("   ")
            && t.split_once(" = ").is_some_and(|(lhs, _)| {
                let mut w = lhs.split_whitespace();
                matches!((w.next(), w.next(), w.next()), (Some(ty), Some(v), None) if ty.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && v.starts_with('*') && v[1..].chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
            });
        if is_decl {
            decls.push_str(line);
            decls.push('\n');
        } else {
            tail.push_str(line);
            tail.push('\n');
        }
    }
    let i_field = if has_loop { " uint64_t i;" } else { "" };
    let i_init = if has_loop { " L.i = 0;" } else { "" };
    let i_local = if has_loop { "  uint64_t i = Lc->i;\n" } else { "" };
    let mut out = String::with_capacity(fast.len() + tail.len() + 512);
    // The prototype first: a unit that keeps this glue to itself puts `static ` before the
    // definition text (`fn_unit`), and the definition below then inherits that linkage.
    let _ = writeln!(out, "void drop_{m}({ct} *p);");
    let _ = writeln!(out, "struct dgf_{m} {{ int32_t st; {ct} *p;{i_field} }};");
    let _ = writeln!(out, "static void dgc_{m}(struct dgf_{m} *Lc);");
    let _ = writeln!(
        out,
        "static inline __attribute__((always_inline)) void dgw_{m}(struct dgf_{m} *Lc) {{ if (__builtin_expect(Lc->st != 0, 0)) dgc_{m}(Lc); }}"
    );
    let _ = writeln!(out, "void drop_{m}({ct} *p) {{\n  struct dgf_{m} L __attribute__((cleanup(dgw_{m})));\n  L.st = 0; L.p = p;{i_init}");
    out.push_str(&fast);
    out.push_str("  L.st = 0;\n}\n");
    let _ = writeln!(out, "static void dgc_{m}(struct dgf_{m} *Lc) {{\n  {ct} *p = Lc->p;\n{i_local}{decls}  switch (Lc->st) {{");
    for k in 1..=stmts.len() {
        let _ = writeln!(out, "  case {k}: goto r{k};");
    }
    out.push_str("  default: return;\n  }\n");
    out.push_str(&tail);
    out.push_str("}\n");
    Some(out)
}

/// True when glue text after a drop statement runs nothing more: only closing braces,
/// `break;` and the switch labels after it (`case N:`, `default:`).
fn nothing_after(rest: &str) -> bool {
    let mut s: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    s = s.replace("break;", "").replace("default:", "");
    while let Some(i) = s.find("case") {
        let digits = s[i + 4..].chars().take_while(char::is_ascii_digit).count();
        if digits == 0 || s.as_bytes().get(i + 4 + digits) != Some(&b':') {
            return false;
        }
        s.replace_range(i..i + 4 + digits + 1, "");
    }
    s.chars().all(|c| c == '}' || c == ';')
}

/// The blocks of `body` that are cleanup: every block reachable from a
/// call's unwind successor or a statement's cleanup entry. Lowering
/// builds them as ladders that only reach each other.
pub(super) fn cleanup_blocks(body: &Body) -> Vec<bool> {
    let mut seen = vec![false; body.blocks.len()];
    let mut stack: Vec<usize> = body.stmt_unwind.iter().map(|&(_, _, u)| u.0 as usize).collect();
    for b in &body.blocks {
        if let Terminator::Call { unwind: Some(u), .. } = b.term {
            stack.push(u.0 as usize);
        }
    }
    while let Some(b) = stack.pop() {
        if std::mem::replace(&mut seen[b], true) {
            continue;
        }
        stack.extend(succ(&body.blocks[b].term).into_iter().map(|s| s.0 as usize));
    }
    seen
}

/// Calls that can panic or run user code, by name prefix. A runtime
/// helper that panics returns a safe placeholder in unwind mode, so
/// only its statement-level callers need the check right after it.
const CHECKED_PREFIXES: &[&str] = &["lust_", "drop_", "clone_", "eq_", "cmp_", "dbg_", "disp_", "default_"];

/// Wraps every call statement `name(..);` in `text` whose name has a
/// checked prefix as `{ name(..); <check> }`, so the code after a
/// panicking call in the same C statement block does not run. Calls
/// inside expressions are left alone: the statement's own check
/// follows them. String and char literals are skipped.
pub(super) fn wrap_calls(text: &str, check: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 64);
    let (mut i, mut copied) = (0, 0);
    while i < b.len() {
        match b[i] {
            b'"' | b'\'' => i = skip_lit(b, i),
            c if is_ident(c) && (i == 0 || !is_ident(b[i - 1])) => {
                let start = i;
                while i < b.len() && is_ident(b[i]) {
                    i += 1;
                }
                let name = &text[start..i];
                if !CHECKED_PREFIXES.iter().any(|p| name.starts_with(p)) || name.starts_with("lust_unw_") || !stmt_start(b, start) {
                    continue;
                }
                let mut j = i;
                while j < b.len() && b[j] == b' ' {
                    j += 1;
                }
                if j >= b.len() || b[j] != b'(' {
                    continue;
                }
                let Some(close) = match_paren(b, j) else { continue };
                let mut k = close + 1;
                while k < b.len() && b[k] == b' ' {
                    k += 1;
                }
                if k >= b.len() || b[k] != b';' {
                    continue;
                }
                out.push_str(&text[copied..start]);
                out.push_str("{ ");
                out.push_str(&text[start..=k]);
                out.push(' ');
                out.push_str(check);
                out.push_str(" }");
                copied = k + 1;
                i = k + 1;
            }
            _ => i += 1,
        }
    }
    out.push_str(&text[copied..]);
    out
}

pub(super) fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// The index after the string or char literal that starts at `i`.
pub(super) fn skip_lit(b: &[u8], i: usize) -> usize {
    let q = b[i];
    let mut j = i + 1;
    while j < b.len() && b[j] != q {
        j += if b[j] == b'\\' { 2 } else { 1 };
    }
    j + 1
}

/// The index of the `)` that closes the `(` at `open`.
fn match_paren(b: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut j = open;
    while j < b.len() {
        match b[j] {
            b'"' | b'\'' => {
                j = skip_lit(b, j);
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Whether the token at `at` starts a C statement: after `;`, `{`,
/// `}`, `else`, the condition of `if`/`for`/`while`, or the start.
/// True when `at` follows an enum glue's `case N:` label (and spaces).
fn after_case_label(b: &[u8], at: usize) -> bool {
    let mut j = at;
    while j > 0 && b[j - 1] == b' ' {
        j -= 1;
    }
    if j == 0 || b[j - 1] != b':' {
        return false;
    }
    let mut k = j - 1;
    while k > 0 && b[k - 1].is_ascii_digit() {
        k -= 1;
    }
    k < j - 1 && k >= 5 && &b[k - 5..k] == b"case "
}

fn stmt_start(b: &[u8], at: usize) -> bool {
    let mut j = at;
    while j > 0 && b[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    if j == 0 {
        return true;
    }
    match b[j - 1] {
        b';' | b'{' | b'}' => true,
        b'e' => j >= 4 && &b[j - 4..j] == b"else" && (j == 4 || !is_ident(b[j - 5])),
        b')' => {
            // Back to the matching `(`, then the keyword before it.
            let mut depth = 0usize;
            let mut k = j - 1;
            loop {
                match b[k] {
                    b')' => depth += 1,
                    b'(' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                if k == 0 {
                    return false;
                }
                k -= 1;
            }
            while k > 0 && b[k - 1] == b' ' {
                k -= 1;
            }
            let end = k;
            while k > 0 && is_ident(b[k - 1]) {
                k -= 1;
            }
            matches!(&b[k..end], b"if" | b"for" | b"while")
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{frame_glue, wrap_calls};

    #[test]
    fn frame_glue_skips_a_frame_with_nothing_to_clean() {
        let framed = |body: &str| frame_glue("T_", "T", body).expect("covered").contains("struct dgf_T_");
        // One drop and nothing after it: plain glue, which units can inline.
        let opt = "  if (disc_O_X_(p)) drop_X_(&(*p).u.v1.f0);\n";
        assert_eq!(frame_glue("T_", "T", opt).as_deref(), Some(format!("void drop_T_(T *p) {{\n{opt}}}\n").as_str()));
        assert!(!framed("  ((void (*)(T *))f_0123456789abcdef)(p);\n"));
        assert!(!framed("  switch (disc_E_(p)) {\n  case 0: drop_X_(&p->u.v0.f0); break;\n  case 1: break;\n  default: break;\n  }\n"));
        // A drop with more after it keeps the frame: the rest runs as cleanup.
        assert!(framed("  drop_X_(&p->f0);\n  drop_Y_(&p->f1);\n"));
        assert!(framed("  ((void (*)(T *))f_0123456789abcdef)(p);\n  drop_X_(&p->f0);\n"));
        assert!(framed("  drop_X_(*p);\n  lust_dealloc(*p, 8, 8);\n"));
        assert!(framed("  switch (disc_E_(p)) {\n  case 0: drop_X_(&p->u.v0.f0); break;\n  case 1: lust_x(p); break;\n  }\n"));
        assert!(framed("  for (uint64_t i = 0; i < p->len; i++) drop_X_(&((X *)p->ptr)[i]);\n"));
    }

    #[test]
    fn wraps_statement_calls_only() {
        let c = "X;";
        assert_eq!(wrap_calls("  lust_bounds(a, b, 3u);\n", c), "  { lust_bounds(a, b, 3u); X; }\n");
        assert_eq!(wrap_calls("  if (x) lust_panic_cstr(\"a (b\", 1u); y = 2;\n", c), "  if (x) { lust_panic_cstr(\"a (b\", 1u); X; } y = 2;\n");
        assert_eq!(wrap_calls("  t = lust_add_i32(a, b, 1u);\n", c), "  t = lust_add_i32(a, b, 1u);\n");
        assert_eq!(wrap_calls("  (void)lust_x(a);\n", c), "  (void)lust_x(a);\n");
        assert_eq!(wrap_calls("  else drop_S(&p);", c), "  else { drop_S(&p); X; }");
        assert_eq!(wrap_calls("  lust_unw_take(&ex);", c), "  lust_unw_take(&ex);");
        assert_eq!(wrap_calls("  for (i = 0; i < n; i++) drop_T(&v[i]);", c), "  for (i = 0; i < n; i++) { drop_T(&v[i]); X; }");
    }
}
