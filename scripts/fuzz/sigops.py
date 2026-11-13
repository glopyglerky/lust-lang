# Signature-level edit ops for the edit fuzzer (sigedit PLAN, P1.1).
#
# Each op takes the crate's files (rel path -> text) and returns the
# states to visit in order, the last one being the files it started
# from, so a pulse leaves the main edit stream as it was:
#   sig_where      a `where` clause on a fn (true, false, or generic bound)
#   sig_param      a fn parameter added, or its type changed
#   sig_ret        a fn return type changed (or `-> ()` written out)
#   rpit_bound     a bound added to an `-> impl Trait` return type
#   derive_drop    one trait out of a `#[derive(..)]`
#   derive_add     one trait into a `#[derive(..)]`
#   alias_rename   `use a::f` becomes `use a::f as g` (uses follow), or a
#                  new `use crate::m::f as g` in another file
#   reexport_glob  `pub use crate::m::*;` in another module
#   rpit_chain     Sol's case: f's return type changes, an unchanged
#                  `g() -> impl Trait { f() }` hides it, a caller needs an
#                  auto trait from g()
#   const_closure  Sol's case: `static P: fn() -> u32 = make();` with
#                  `const fn make() -> fn() -> u32 { f }`, then a `where`
#                  clause on f
#   sig_generic    the header delta's own op (driver/kept_delta.rs takes
#                  only programs with user generics): a generic fn first,
#                  sometimes an RPIT user or a const closure of a fresh fn,
#                  then two header edits in a row, back, and the files
#   sig_impl       method header edits the header delta takes in impls: a
#                  user trait's impl (concrete and generic), and an inherent
#                  impl with an associated const; callers through the type,
#                  a generic bound, UFCS and the trait path; two edits, back
#   sig_apit       a fn with an `impl Trait` param (APIT): header edits that
#                  keep the APITs (the item patch keeps their lowering) and
#                  ones that change, move, or add one (a full resolve)
#   sig_umacro     a fn whose body calls a user macro: header edits with the
#                  same body (the item patch reuses the body's expansion),
#                  then a macro input or macro definition edit
#   sig_umbody     body edits in fns that call user macros (one from the
#                  same module file, one `#[macro_export]` from the root by
#                  `crate::` path): the body patch expands the file again
#                  (`expand_by_file`); errors, hygiene, refused shapes
#   sig_implmac    an impl whose methods are partly made by an impl-level
#                  macro call, with a nested fn in another method: header
#                  edits of a written method (the item patch skips the made
#                  ones and refolds the nested fn's hash)
#   sig_tdefault   trait default bodies (templates after the AST fns) that
#                  call a fn and use a derived type: header edits of the fn,
#                  and a derive drop of a trait a default body uses
#   sig_dyn        a fn with a `dyn` type in its header: header edits that
#                  keep the traits `dyn` types name (the header delta takes
#                  them) and ones that add or drop one (eager vtables move)
#   sig_samename   inherent methods of the same name on two Adts, a free fn
#                  and a trait method of that name: header edits of one of
#                  them (the header delta's readers key by Adt and def)
#   sig_constinit  const and static initializers (items, an impl const, a
#                  const in a fn body) that call a const fn, coerce a fn to
#                  a fn pointer, or copy a derived type: header edits of the
#                  fns and derive drops (const_body::check_dirty's keyed walk)
# `External` adds verdicts from rustc 1.99 and a pinned baseline lust for
# the op states (`--rustc`, `--baseline`): a state where the warm answer
# disagrees with rustc fails unless the baseline gives the same answer
# (a known gap, counted, not a failure); with no baseline, only a false
# accept fails.
import os, re, subprocess, tempfile

OPS = ["sig_where", "sig_param", "sig_ret", "rpit_bound", "derive_drop", "derive_add",
       "alias_rename", "reexport_glob", "rpit_chain", "const_closure", "sig_generic", "sig_impl",
       "sig_apit", "sig_umacro", "sig_umbody", "sig_implmac", "sig_tdefault", "sig_dyn", "sig_samename",
       "sig_constinit"]

# One-line fn headers: indent, head up to the name, generics, params,
# return type. A header that already has a `where` is left alone.
FN_HEAD = re.compile(r"^(?P<ind>[ \t]*)(?P<pre>(?:pub(?:\([\w:]+\))? )?(?:const )?(?:unsafe )?)fn (?P<name>\w+)"
                     r"(?P<gen><[^>{}]*>)?\((?P<params>[^()]*(?:\([^()]*\)[^()]*)*)\)"
                     r"(?: -> (?P<ret>[^{]+?))? \{$", re.M)
DERIVE = re.compile(r"#\[derive\(([^)]*)\)\]")
SINGLE_USE = re.compile(r"^(?P<ind>[ \t]*)(?P<vis>pub(?:\([\w:]+\))? )?use (?P<path>(?:crate|super|self)(?:::\w+)*)::(?P<name>[a-z_]\w*);$", re.M)
PUB_FN = re.compile(r"^pub fn (\w+)", re.M)
DERIVABLE = ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default", "Hash", "PartialOrd"]


def _heads(fs):
    return [(rel, m) for rel in sorted(fs) for m in FN_HEAD.finditer(fs[rel]) if m.group("name") != "main"]


def _swap(fs, rel, s, e, new):
    out = dict(fs)
    out[rel] = fs[rel][:s] + new + fs[rel][e:]
    return out


def _header(m, gen=None, params=None, ret=None, where=""):
    g = m.group("gen") if gen is None else gen
    p = m.group("params") if params is None else params
    r = m.group("ret") if ret is None else ret
    return f"{m.group('ind')}{m.group('pre')}fn {m.group('name')}{g or ''}({p}){' -> ' + r if r else ''}{where} {{"


def _mod_path(rel):
    """The module path of a file in a multi crate (`a/inner.rs` -> `crate::a::inner`)."""
    if rel in ("main.rs", "crate.rs", "lib.rs"):
        return "crate"
    parts = rel[:-3].split("/")
    if parts[-1] == "mod":
        parts = parts[:-1]
    return "crate::" + "::".join(parts)


def sig_where(rng, fs):
    heads = _heads(fs)
    if not heads:
        return []
    rel, m = rng.choice(heads)
    clauses = [" where u32: Copy", " where String: Copy", " where i64: Clone + Default"]
    params = re.findall(r"<\s*(\w+)", m.group("gen") or "")
    if params:
        clauses += [f" where {params[0]}: Clone", f" where {params[0]}: Default", f" where {params[0]}: Sized"]
    return [_swap(fs, rel, m.start(), m.end(), _header(m, where=rng.choice(clauses))), fs]


def sig_param(rng, fs):
    heads = _heads(fs)
    if not heads:
        return []
    rel, m = rng.choice(heads)
    p = m.group("params")
    typed = list(re.finditer(r"\b(i64|i32|u32|u64|usize)\b", p))
    if typed and rng.random() < 0.6:
        t = rng.choice(typed)
        new_t = rng.choice([x for x in ["i64", "i32", "u32", "u64"] if x != t.group(1)])
        params = p[:t.start()] + new_t + p[t.end():]
    else:
        params = (p + ", " if p.strip() else "") + "_sig: u8"
    return [_swap(fs, rel, m.start(), m.end(), _header(m, params=params)), fs]


def sig_ret(rng, fs):
    heads = _heads(fs)
    if not heads:
        return []
    rel, m = rng.choice(heads)
    r = m.group("ret")
    if r is None:
        ret = "()"
    elif r.startswith("impl "):
        ret = r + " + Sized"
    else:
        ret = rng.choice(["i32", "u64", "Option<" + r + ">", "()", "&'static str", r])
    return [_swap(fs, rel, m.start(), m.end(), _header(m, ret=ret)), fs]


def rpit_bound(rng, fs):
    sites = [(rel, m) for rel, m in _heads(fs) if (m.group("ret") or "").startswith("impl ")]
    if not sites:
        return []
    rel, m = rng.choice(sites)
    ret = m.group("ret") + rng.choice([" + Sized", " + Copy", " + Send", " + Sync", " + 'static", " + Clone"])
    return [_swap(fs, rel, m.start(), m.end(), _header(m, ret=ret)), fs]


def _derive_sites(fs):
    return [(rel, m) for rel in sorted(fs) for m in DERIVE.finditer(fs[rel])]


def derive_drop(rng, fs):
    sites = [(rel, m) for rel, m in _derive_sites(fs) if m.group(1).strip()]
    if not sites:
        return []
    rel, m = rng.choice(sites)
    names = [x.strip() for x in m.group(1).split(",") if x.strip()]
    names.remove(rng.choice(names))
    return [_swap(fs, rel, m.start(), m.end(), f"#[derive({', '.join(names)})]"), fs]


def derive_add(rng, fs):
    sites = _derive_sites(fs)
    if not sites:
        return []
    rel, m = rng.choice(sites)
    names = [x.strip() for x in m.group(1).split(",") if x.strip()]
    extra = [d for d in DERIVABLE if d not in names]
    if not extra:
        return []
    names.insert(rng.randrange(len(names) + 1), rng.choice(extra))
    return [_swap(fs, rel, m.start(), m.end(), f"#[derive({', '.join(names)})]"), fs]


def alias_rename(rng, fs):
    uses = [(rel, m) for rel in sorted(fs) for m in SINGLE_USE.finditer(fs[rel])]
    if uses and rng.random() < 0.7:
        rel, m = rng.choice(uses)
        name = m.group("name")
        alias = name + "_sig"
        text = fs[rel]
        rest = re.sub(rf"\b{name}\b(?=\s*[(!:<])", alias, text[m.end():])
        new = f"{m.group('ind')}{m.group('vis') or ''}use {m.group('path')}::{name} as {alias};"
        out = dict(fs)
        out[rel] = text[:m.start()] + new + rest
        return [out, fs]
    # A new alias of another file's pub fn (unused: a warning both ways).
    defs = [(rel, n) for rel in sorted(fs) for n in PUB_FN.findall(fs[rel])]
    if len(fs) < 2 or not defs:
        return []
    rel, name = rng.choice(defs)
    into = rng.choice([r for r in sorted(fs) if r != rel])
    line = f"#[allow(unused_imports)]\nuse {_mod_path(rel)}::{name} as {name}_alias;\n"
    out = dict(fs)
    out[into] = _after_inner_attrs(fs[into], line)
    return [out, fs]


def _after_inner_attrs(text, line):
    """`text` with `line` inserted after its inner attributes (`#![..]`)."""
    m = re.match(r"(?:\s*#!\[[^\]]*\]\s*\n)*", text)
    k = m.end() if m else 0
    return text[:k] + line + text[k:]


def reexport_glob(rng, fs):
    mods = [r for r in sorted(fs) if _mod_path(r) != "crate"]
    if not mods:
        return []
    src = rng.choice(mods)
    into = rng.choice([r for r in sorted(fs) if r != src])
    line = f"#[allow(unused_imports)]\npub use {_mod_path(src)}::*;\n"
    out = dict(fs)
    out[into] = _after_inner_attrs(fs[into], line)
    return [out, fs]


def _root(fs):
    for r in ("main.rs", "crate.rs"):
        if r in fs:
            return r
    return sorted(fs)[0]


def rpit_chain(rng, fs):
    """Sol's RPIT chain, appended to the root file with fresh names: A
    (passes), B (f's return type changes; the unchanged opaque users must
    see it), back to A, then the files as they were."""
    k = rng.randrange(10**6)
    auto = rng.choice(["Send", "Sync"])
    depth = rng.choice([1, 2])
    trait = rng.choice(["std::fmt::Debug", "Clone"])
    chain = f"fn zs{k}_g() -> impl {trait} {{ zs{k}_f() }}\n"
    top = f"zs{k}_g"
    if depth == 2:
        chain += f"fn zs{k}_h() -> impl {trait} {{ zs{k}_g() }}\n"
        top = f"zs{k}_h"
    def items(ret, val):
        return (f"\nfn zs{k}_f() -> {ret} {{ {val} }}\n{chain}"
                f"fn zs{k}_need<T: {auto}>(t: T) -> T {{ t }}\n"
                f"#[allow(dead_code)]\nfn zs{k}_use() {{ let _ = zs{k}_need({top}()); }}\n")
    root = _root(fs)
    a = dict(fs)
    a[root] = fs[root] + items("u32", "7")
    bad = rng.choice([("std::rc::Rc<u32>", "std::rc::Rc::new(7)"), ("std::cell::Cell<u32>", "std::cell::Cell::new(7)"),
                      ("u64", "7"), ("*const u8", "std::ptr::null()")])
    b = dict(fs)
    b[root] = fs[root] + items(*bad)
    return [a, b, a, fs]


def const_closure(rng, fs):
    """Sol's const-fn closure, appended to the root file: A (passes), B (a
    `where` clause on f, which make() names), back to A, then the files."""
    k = rng.randrange(10**6)
    def items(where_f, where_make):
        return (f"\nfn zc{k}_f() -> u32{where_f} {{ 5 }}\n"
                f"const fn zc{k}_make() -> fn() -> u32{where_make} {{ zc{k}_f }}\n"
                f"static ZC{k}_P: fn() -> u32 = zc{k}_make();\n"
                f"#[allow(dead_code)]\nfn zc{k}_use() -> u32 {{ let p = ZC{k}_P; p() }}\n")
    root = _root(fs)
    a = dict(fs)
    a[root] = fs[root] + items("", "")
    b = dict(fs)
    b[root] = fs[root] + rng.choice([items(" where u32: Copy", ""), items("", " where u32: Copy")])
    return [a, b, a, fs]


def sig_generic(rng, fs):
    """A: the files with a generic fn (and maybe a fresh fn `f` that an
    unchanged `-> impl Debug` body or a const fn names) at the end of the
    root file; then one header edit on A, a second on top of it, back to
    A, and the files. The edits pick `f` half the time when it exists."""
    k = rng.randrange(10**6)
    extra = rng.choice(["", "rpit", "const"])
    items = f"\n#[allow(dead_code)]\nfn zg{k}_id<T: Clone>(t: T) -> T {{ t.clone() }}\n"
    if extra == "rpit":
        auto = rng.choice(["Send", "Sync"])
        items += (f"fn zg{k}_f(x: u32) -> u32 {{\n    x\n}}\n"
                  f"fn zg{k}_g() -> impl std::fmt::Debug {{ zg{k}_f(1) }}\n"
                  f"fn zg{k}_need<T: {auto}>(t: T) -> T {{ t }}\n"
                  f"#[allow(dead_code)]\nfn zg{k}_use() {{ let _ = zg{k}_need(zg{k}_g()); }}\n")
    elif extra == "const":
        items += (f"fn zg{k}_f(x: u32) -> u32 {{\n    x\n}}\n"
                  f"const fn zg{k}_make() -> fn(u32) -> u32 {{ zg{k}_f }}\n"
                  f"static ZG{k}_P: fn(u32) -> u32 = zg{k}_make();\n"
                  f"#[allow(dead_code)]\nfn zg{k}_use() -> u32 {{ let p = ZG{k}_P; p(2) }}\n")
    root = _root(fs)
    a = dict(fs)
    a[root] = fs[root] + items

    def edit(cur):
        heads = _heads(cur)
        own = [(rel, m) for rel, m in heads if m.group("name") == f"zg{k}_f"]
        if own and rng.random() < 0.5:
            rel, m = own[0]
            new = rng.choice([_header(m, where=" where u32: Copy"), _header(m, where=" where String: Copy"),
                              _header(m, params="x: u64"), _header(m, ret="u64"), _header(m, params="x: u32, _y: u8")])
            return _swap(cur, rel, m.start(), m.end(), new)
        op = rng.choice([sig_where, sig_param, sig_ret])
        states = op(rng, cur)
        return states[0] if states else cur

    s1 = edit(a)
    s2 = edit(s1)
    return [a, s1, s2, a, fs]


def sig_impl(rng, fs):
    """A: the files with a user trait, a concrete and a generic impl of it,
    an inherent impl with an associated const, and their callers at the
    end of the root file; then one method header edit, a second on top,
    back to A, and the files."""
    k = rng.randrange(10**6)
    items = (f"\ntrait Zt{k} {{\n    type Out: std::fmt::Debug;\n    fn zm{k}(&self, x: u32) -> u32;\n    fn zn{k}(&self) -> Self::Out;\n}}\n"
             f"struct Zs{k}(u32);\n"
             f"impl Zs{k} {{\n    const C: u32 = 3;\n    fn zh{k}(&self, x: u32) -> u32 {{\n        x + Self::C + self.0\n    }}\n}}\n"
             f"impl Zt{k} for Zs{k} {{\n    type Out = u32;\n    fn zm{k}(&self, x: u32) -> u32 {{\n        x + self.0\n    }}\n"
             f"    fn zn{k}(&self) -> Self::Out {{\n        self.0\n    }}\n}}\n"
             f"impl<T: Clone> Zt{k} for Vec<T> {{\n    type Out = usize;\n    fn zm{k}(&self, x: u32) -> u32 {{\n        x\n    }}\n"
             f"    fn zn{k}(&self) -> usize {{\n        self.len()\n    }}\n}}\n"
             f"fn zt{k}_gen<T: Zt{k}>(t: &T) -> u32 {{ t.zm{k}(1) }}\n"
             f"#[allow(dead_code)]\nfn zt{k}_use() -> u32 {{\n    let s = Zs{k}(2);\n"
             f"    let _ = format!(\"{{:?}}{{:?}}\", s.zn{k}(), vec![1u8].zn{k}());\n"
             f"    s.zm{k}(3) + zt{k}_gen(&s) + Zt{k}::zm{k}(&vec![1u8], 4) + s.zh{k}(1) + <Zs{k} as Zt{k}>::zm{k}(&s, 0)\n}}\n")
    root = _root(fs)
    a = dict(fs)
    a[root] = fs[root] + items

    def edit(cur):
        own = [(rel, m) for rel, m in _heads(cur) if m.group("name") in (f"zm{k}", f"zn{k}", f"zh{k}")]
        if not own:
            return cur
        rel, m = rng.choice(own)
        if m.group("name") == f"zn{k}":
            new = rng.choice([_header(m, ret="u32"), _header(m, ret="usize"), _header(m, ret="Self::Out"),
                              _header(m, where=" where u32: Copy"), _header(m, ret="u64")])
        else:
            new = rng.choice([_header(m, where=" where u32: Copy"), _header(m, where=" where String: Copy"),
                              _header(m, params=m.group("params").replace("x: u32", "mut x: u32")), _header(m, params=m.group("params").replace("x: u32", "x: u64")),
                              _header(m, ret="u64"), _header(m, gen="<'a>", params=m.group("params").replace("&self", "&'a self"))])
        return _swap(cur, rel, m.start(), m.end(), new)

    s1 = edit(a)
    s2 = edit(s1)
    return [a, s1, s2, a, fs]
def _header_walk(rng, fs, items, head, edits, extra=()):
    """A: `items` at the end of the root file; then two header edits in a
    row (`head` replaced by a pick of `edits`, the second on the first's
    text), back to A; with `extra`, one other text swap, a header edit on
    top of it, back to A; then the files."""
    root = _root(fs)
    a = dict(fs)
    a[root] = fs[root] + items
    def swap(cur, old, new):
        out = dict(cur)
        out[root] = cur[root].replace(old, new, 1)
        return out
    h1 = rng.choice(edits)
    s1 = swap(a, head, h1)
    s2 = swap(s1, h1, rng.choice([e for e in edits if e != h1] or edits))
    states = [a, s1, s2, a]
    if extra:
        old, new = rng.choice(extra)
        b = swap(a, old, new)
        states += [b, swap(b, head, rng.choice(edits)), a]
    return states + [fs]


def sig_apit(rng, fs):
    """A fn with an APIT (sometimes after generics whose bound holds
    parens) and its caller; header edits that keep the APIT or change,
    move, or add one."""
    k = rng.randrange(10**6)
    gen = rng.choice(["", "<F: Fn(u8) -> u8>"])
    fparam = ", f: F" if gen else ""
    farg = ", |b| b + 1" if gen else ""
    apit = "mut test: impl FnMut(u32) -> bool"
    def h(params=None, ret="bool", where=""):
        p = params if params is not None else f"xs: &[u32], {apit}{fparam}"
        return f"fn za{k}_any{gen}({p}) -> {ret}{where} {{"
    head = h()
    items = (f"\n#[allow(dead_code)]\n{head}\n    xs.iter().any(|&x| test(x))\n}}\n"
             f"#[allow(dead_code)]\nfn za{k}_use() -> bool {{ za{k}_any(&[1, 2], |x| x > 1{farg}) }}\n")
    edits = [h(where=" where u32: Copy"), h(where=" where String: Copy"), h(where=" where i64: Clone + Default"),
             h(params=f"xs: &[u32], {apit}{fparam}, _k: u8"), h(params=f"xs: &[u64], {apit}{fparam}"), h(ret="Option<bool>"),
             # APIT changes: a full resolve.
             h(params=f"xs: &[u32], mut test: impl Fn(u32) -> bool{fparam}"),
             h(params=f"xs: &[u32], {apit} + Copy{fparam}"),
             h(params=f"{apit}, xs: &[u32]{fparam}"),
             h(params=f"xs: &[u32], {apit}{fparam}, _y: impl Copy")]
    return _header_walk(rng, fs, items, head, edits)


def sig_umacro(rng, fs):
    """A fn whose body calls a user macro, and one whose macro binds a
    hygienic local; header edits with the same body, then a macro input
    or macro definition edit with a header edit on top."""
    k = rng.randrange(10**6)
    mdef = "{ vec![$($x.to_string()),*] }"
    items = (f"\nmacro_rules! zu{k}_args {{\n    ($($x:expr),* $(,)?) => {mdef};\n}}\n"
             f"macro_rules! zu{k}_tw {{\n    ($e:expr) => {{{{ let t = $e; t + t }}}};\n}}\n"
             f"#[allow(dead_code)]\nfn zu{k}_chain(d: &str, n: usize) -> Vec<String> {{\n    let mut v = zu{k}_args![d, n];\n"
             f"    v.push(String::new());\n    v\n}}\n"
             f"#[allow(dead_code)]\nfn zu{k}_twice(t: i64) -> i64 {{\n    zu{k}_tw!(t)\n}}\n"
             f"#[allow(dead_code)]\nfn zu{k}_use() -> i64 {{ zu{k}_chain(\"a\", 2).len() as i64 + zu{k}_twice(3) }}\n")
    if rng.random() < 0.3:
        head = f"fn zu{k}_twice(t: i64) -> i64 {{"
        edits = [f"fn zu{k}_twice(t: i64) -> i64 where i64: Copy {{", f"fn zu{k}_twice(t: i64) -> i64 where String: Copy {{",
                 f"fn zu{k}_twice(t: i32) -> i32 {{", f"fn zu{k}_twice(u: i64) -> i64 {{", f"fn zu{k}_twice(t: u8) -> i64 {{"]
    else:
        c = f"fn zu{k}_chain"
        head = f"{c}(d: &str, n: usize) -> Vec<String> {{"
        edits = [f"{c}(d: &str, n: usize) -> Vec<String> where u32: Copy {{", f"{c}(d: &str, n: usize) -> Vec<String> where String: Copy {{",
                 f"{c}(d: &str, n: usize, _k: u8) -> Vec<String> {{", f"{c}(e: &str, n: usize) -> Vec<String> {{",
                 f"{c}(d: &str, n: u8) -> Vec<String> {{", f"{c}(d: &String, n: usize) -> Vec<String> {{"]
    extra = [(f"zu{k}_args![d, n]", f"zu{k}_args![d, n, 1]"),
             (mdef, "{ [$(format!(\"{}\", $x)),*].to_vec() }"),
             (mdef, "{ vec![$($x),*] }")]
    return _header_walk(rng, fs, items, head, edits, extra)


def sig_umbody(rng, fs):
    """Fns whose bodies call user macros, in a module file when the crate
    has one (the body patch refuses the root file): a textual macro of
    that file and a root `#[macro_export]` one whose `$body` runs twice
    (once in a closure). A: the items; then two body edits in a row (the
    second on the first's text when its anchor stands), back to A, then
    the files."""
    k = rng.randrange(10**6)
    root = _root(fs)
    mods = [r for r in sorted(fs) if r != root and r.endswith(".rs")]
    tgt = rng.choice(mods) if mods else root
    tw, wv = f"zv{k}_tw", f"zv{k}_with"
    a = dict(fs)
    a[root] = fs[root] + (f"\n#[macro_export]\nmacro_rules! {wv} {{\n    ($v:expr, |$d:ident| $body:expr) => {{{{\n"
                          f"        let v__: i64 = $v;\n        match v__ {{\n            0 => {{\n                let $d = v__;\n"
                          f"                $body\n            }}\n            _ => (|$d: i64| $body)(v__),\n        }}\n    }}}};\n}}\n")
    a[tgt] = a[tgt] + (f"\nmacro_rules! {tw} {{\n    ($e:expr) => {{{{ let t = $e; t + t }}}};\n}}\n"
                       f"#[allow(dead_code)]\nfn zv{k}_a(a: i64) -> i64 {{\n    let v = {tw}!(a);\n    v + 1\n}}\n"
                       f"#[allow(dead_code)]\nfn zv{k}_b(n: i64) -> i64 {{\n    crate::{wv}!(n, |d| d + 1)\n}}\n")
    va, vb, tail = f"let v = {tw}!(a);", f"crate::{wv}!(n, |d| d + 1)", "    v + 1\n"
    edits = [(va, f"let v = {tw}!(a + 2);"), (va, f"let v = {tw}!(a);\n    let _w: u32 = {tw}!(1u32);"),
             (va, f"let v: u32 = {tw}!(a);"), (va, f"let t = 3;\n    let v = {tw}!(a) + t;"),
             (va, f"let v = {tw}!(a) + {tw}!({tw}!(a));"), (va, f"let v = {tw}!(a) + zv{k}_nope!(1);"),
             (va, f"macro_rules! zv{k}_in {{\n        () => {{ 4 }};\n    }}\n    let v = {tw}!(a) + zv{k}_in!();"),
             (vb, f"crate::{wv}!(n, |d| d * 2 + {tw}!(d))"), (vb, f"crate::{wv}!(n, |d| d + t)"),
             (vb, f"crate::{wv}!(n, |d| d + 1) + {tw}!(n as u8) as i64"),
             (tail, f"    v + format!(\"{{}}\", {tw}!(1)).len() as i64\n"), (tail, "    v + line!() as i64\n")]
    def swap(cur, old, new):
        out = dict(cur)
        out[tgt] = cur[tgt].replace(old, new, 1)
        return out
    e1, e2 = rng.choice(edits), rng.choice(edits)
    s1 = swap(a, *e1)
    s2 = swap(s1 if e2[0] in s1[tgt] else a, *e2)
    return [a, s1, s2, a, fs]


def sig_implmac(rng, fs):
    """An impl with two methods an impl-level macro makes, one method with
    a nested fn, and one plain method; header edits of the plain one
    (`_header_walk`), and a body edit of the nested fn on top."""
    k = rng.randrange(10**6)
    c = f"    pub fn zi{k}_sum"
    head = f"{c}(&self, k: i64) -> i64 {{"
    items = (f"\nmacro_rules! zi{k}_get {{\n    ($n:ident, $m:expr) => {{\n        pub fn $n(&self) -> i64 {{\n            self.0 * $m\n"
             f"        }}\n    }};\n}}\n#[allow(dead_code)]\npub struct Zi{k}(i64);\n#[allow(dead_code)]\nimpl Zi{k} {{\n"
             f"    zi{k}_get!(zi{k}_two, 2);\n    pub fn zi{k}_nest(&self) -> i64 {{\n        fn zi{k}_in(x: i64) -> i64 {{\n"
             f"            x + 1\n        }}\n        zi{k}_in(self.0)\n    }}\n    zi{k}_get!(zi{k}_three, 3);\n"
             f"{head}\n        self.zi{k}_two() + self.zi{k}_three() + k\n    }}\n}}\n"
             f"#[allow(dead_code)]\nfn zi{k}_use() -> i64 {{\n    let w = Zi{k}(2);\n    w.zi{k}_sum(1) + w.zi{k}_nest()\n}}\n")
    edits = [f"{c}(&self, k: i64) -> i64 where u32: Copy {{", f"{c}(&self, k: i64) -> i64 where String: Copy {{",
             f"{c}(&self, k: u8) -> i64 {{", f"{c}(&self, k: i64, _e: u8) -> i64 {{", f"{c}(&self, k: i64) -> u64 {{"]
    extra = [("            x + 1\n", "            x + 2\n"), (f"zi{k}_get!(zi{k}_two, 2);", f"zi{k}_get!(zi{k}_two, 4);")]
    return _header_walk(rng, fs, items, head, edits, extra)


def sig_tdefault(rng, fs):
    """A fn `f`, a derived struct, and a trait whose default bodies (a
    method with a closure, an associated fn) call f and use the struct's
    Copy, Debug and Default; then header edits of f, and a derive drop
    with a header edit on top (`_header_walk`)."""
    k = rng.randrange(10**6)
    head = f"fn zd{k}_f(x: u32) -> u32 {{"
    derive = "#[derive(Clone, Copy, Debug, Default)]"
    items = (f"\n{head}\n    x\n}}\n{derive}\nstruct Zq{k}(u32);\n"
             f"trait Zd{k} {{\n    fn zd{k}_m(&self, q: Zq{k}) -> u32 {{\n        let c = || zd{k}_f(q.0);\n        let (a, b) = (q, q);\n"
             f"        c() + a.0 + b.0 + format!(\"{{:?}}\", q).len() as u32\n    }}\n"
             f"    fn zd{k}_s() -> u32 {{\n        let d: Zq{k} = Default::default();\n        zd{k}_f(d.0)\n    }}\n}}\n"
             f"impl Zd{k} for u8 {{}}\n"
             f"#[allow(dead_code)]\nfn zd{k}_use() -> u32 {{\n    1u8.zd{k}_m(Zq{k}(2)) + <u8 as Zd{k}>::zd{k}_s()\n}}\n")
    edits = [f"fn zd{k}_f(x: u64) -> u32 {{", f"fn zd{k}_f(x: u32) -> u64 {{", f"fn zd{k}_f(x: &u32) -> u32 {{",
             f"fn zd{k}_f(x: u32, _y: u8) -> u32 {{", f"fn zd{k}_f(x: u32) -> u32 where u32: Copy {{",
             f"const fn zd{k}_f(x: u32) -> u32 {{"]
    drops = [(derive, derive.replace(t, "")) for t in ("Copy, ", "Debug, ", ", Default", "Clone, Copy, ")]
    return _header_walk(rng, fs, items, head, edits, drops)


def sig_dyn(rng, fs):
    """Two traits, one only some edits name in a `dyn`; a fn with a `dyn`
    param, a const coerced to `dyn` in a body (eager vtables), and their
    callers; header edits of the `dyn` fn."""
    k = rng.randrange(10**6)
    items = (f"\ntrait Zd{k} {{\n    fn zd{k}(&self) -> u32;\n}}\n#[allow(dead_code)]\ntrait Ze{k} {{\n    fn ze{k}(&self) -> u32;\n}}\n"
             f"struct Zds{k}(u32);\nimpl Zd{k} for Zds{k} {{\n    fn zd{k}(&self) -> u32 {{\n        self.0\n    }}\n}}\n"
             f"impl Ze{k} for Zds{k} {{\n    fn ze{k}(&self) -> u32 {{\n        self.0 + 1\n    }}\n}}\n"
             f"const ZDC{k}: Zds{k} = Zds{k}(5);\n"
             f"#[allow(dead_code)]\nfn zd{k}_visit(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u32 {{\n    f(x)\n}}\n"
             f"#[allow(dead_code)]\nfn zd{k}_show(t: &dyn Zd{k}) -> u32 {{\n    t.zd{k}()\n}}\n"
             f"#[allow(dead_code)]\nfn zd{k}_wrap<T: Copy>(t: T) -> T {{ t }}\n"
             f"#[allow(dead_code)]\nfn zd{k}_use() -> u32 {{\n    let c: &dyn Zd{k} = &ZDC{k};\n    let mut n = 0;\n"
             f"    zd{k}_visit(&mut |x| {{ n += x; x }}, 2) + zd{k}_show(&Zds{k}(3)) + c.zd{k}() + zd{k}_wrap(n)\n}}\n")
    if rng.random() < 0.5:
        c = f"fn zd{k}_visit"
        head = f"{c}(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u32 {{"
        edits = [f"{c}(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u32 where u32: Copy {{",
                 f"{c}(f: &mut (dyn FnMut(u32) -> u32 + Send), x: u32) -> u32 {{",
                 f"{c}(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u64 {{",
                 f"{c}(f: &mut dyn Fn(u32) -> u32, x: u32) -> u32 {{",
                 f"{c}(f: &mut dyn FnMut(u32) -> u32, x: u32, _e: Option<&dyn Ze{k}>) -> u32 {{",
                 f"{c}(f: &mut dyn FnMut(u32) -> u32, x: u32) -> u32 where String: Copy {{"]
    else:
        c = f"fn zd{k}_show"
        head = f"{c}(t: &dyn Zd{k}) -> u32 {{"
        edits = [f"{c}(t: &dyn Zd{k}) -> u32 where u32: Copy {{", f"{c}(t: &(dyn Zd{k} + 'static)) -> u32 {{",
                 f"{c}(t: &dyn Ze{k}) -> u32 {{", f"{c}(t: &Zds{k}) -> u32 {{", f"{c}(t: &dyn Zd{k}) -> u64 {{",
                 f"{c}(t: &dyn Zd{k}, _e: Option<Box<dyn Ze{k}>>) -> u32 {{"]
    return _header_walk(rng, fs, items, head, edits)


def sig_samename(rng, fs):
    """Two Adts with an inherent method of one name, a free fn and a trait
    method of that name; callers by method, UFCS, autoderef and generics;
    header edits of one of the methods or of the free fn."""
    k = rng.randrange(10**6)
    g = f"zg{k}"
    items = (f"\nmod zgm{k} {{\n    pub fn {g}(x: u32) -> u32 {{\n        x + 1\n    }}\n}}\n"
             f"#[derive(Clone, Copy)]\nstruct Zga{k}(u32);\nstruct Zgb{k}<T>(T);\ntrait Zgt{k} {{\n    fn {g}(&self) -> u32;\n}}\n"
             f"impl Zga{k} {{\n    fn {g}(&self) -> u32 {{\n        self.0\n    }}\n}}\n"
             f"impl<T: Copy + Into<u32>> Zgb{k}<T> {{\n    fn {g}(&self) -> u32 {{\n        self.0.into()\n    }}\n}}\n"
             f"impl Zgt{k} for u8 {{\n    fn {g}(&self) -> u32 {{\n        7\n    }}\n}}\n"
             f"#[allow(dead_code)]\nfn {g}_use(a: &Zga{k}, b: Box<Zga{k}>) -> u32 {{\n"
             f"    a.{g}() + Zga{k}::{g}(a) + b.{g}() + Zgb{k}(3u8).{g}() + Zgb{k}::<u8>::{g}(&Zgb{k}(1)) + 5u8.{g}() + zgm{k}::{g}(1)\n}}\n")
    heads = {f"    fn {g}(&self) -> u32 {{\n        self.0\n": [
                f"    fn {g}(&self) -> u32 where u32: Copy {{\n        self.0\n", f"    fn {g}(&self) -> u64 {{\n        self.0 as u64\n",
                f"    fn {g}(self) -> u32 {{\n        self.0\n", f"    fn {g}(&self, _k: u8) -> u32 {{\n        self.0\n"],
             f"    fn {g}(&self) -> u32 {{\n        self.0.into()\n": [
                f"    fn {g}(&self) -> u32 where T: Copy {{\n        self.0.into()\n", f"    fn {g}(&self) -> u64 {{\n        let x: u32 = self.0.into();\n        x as u64\n"],
             f"    pub fn {g}(x: u32) -> u32 {{\n": [
                f"    pub fn {g}(x: u32) -> u32 where u32: Copy {{\n", f"    pub fn {g}(x: u64) -> u32 {{\n        let x = x as u32;\n",
                f"    pub fn {g}(x: String) -> u32 {{\n        let x = x.len() as u32;\n"]}
    head = rng.choice(sorted(heads))
    return _header_walk(rng, fs, items, head, heads[head])


def sig_constinit(rng, fs):
    """A const fn, a plain fn, and a derived struct that const and static
    initializers read (a const fn call, fn pointer coercions, a copy), an
    unrelated const, an impl const, a const in a fn body, and a fn no const
    names; header edits of one fn, and a derive drop with a header edit on
    top."""
    k = rng.randrange(10**6)
    derive = "#[derive(Clone, Copy, Debug)]"
    fhead = f"const fn zi{k}_f(x: u32) -> u32 {{"
    hhead = f"fn zi{k}_h(x: u32) -> u32 {{"
    ohead = f"fn zi{k}_o(x: u32) -> u32 {{"
    items = (f"\n{derive}\nstruct Zk{k}(u32);\n{fhead}\n    x + 1\n}}\n{hhead}\n    x * 2\n}}\n{ohead}\n    x\n}}\n"
             f"const ZI{k}_A: u32 = zi{k}_f(1);\n"
             f"const ZI{k}_P: (Zk{k}, Zk{k}) = {{\n    let q = Zk{k}(2);\n    (q, q)\n}};\n"
             f"const ZI{k}_FP: fn(u32) -> u32 = zi{k}_h;\n"
             f"static ZI{k}_T: [fn(u32) -> u32; 2] = [zi{k}_h, zi{k}_h];\n"
             f"const ZI{k}_Q: u32 = 9;\n"
             f"struct Zw{k};\nimpl Zw{k} {{\n    const D: u32 = zi{k}_f(2) + 1;\n}}\n"
             f"#[allow(dead_code)]\nfn zi{k}_use() -> u32 {{\n    const N: fn(u32) -> u32 = zi{k}_h;\n"
             f"    ZI{k}_A + ZI{k}_P.0.0 + ZI{k}_P.1.0 + ZI{k}_FP(1) + ZI{k}_T[1](2) + ZI{k}_Q + Zw{k}::D + N(3) + zi{k}_o(4)\n}}\n")
    pick = rng.randrange(3)
    if pick == 0:
        head = fhead
        edits = [f"const fn zi{k}_f(x: u32) -> u64 {{", f"const fn zi{k}_f(x: u64) -> u32 {{", f"fn zi{k}_f(x: u32) -> u32 {{",
                 f"const fn zi{k}_f(x: u32) -> u32 where u32: Copy {{", f"const fn zi{k}_f(x: u32, ) -> u32 {{",
                 f"const fn zi{k}_f(x: u32, _y: u8) -> u32 {{"]
    elif pick == 1:
        # No const names zi_o: the header delta takes its edits, and the
        # const bodies take the keyed walk.
        head = ohead
        edits = [f"fn zi{k}_o(x: u32) -> u64 {{", f"fn zi{k}_o(x: u64) -> u32 {{", f"fn zi{k}_o(x: u32, ) -> u32 {{",
                 f"fn zi{k}_o(x: u32) -> u32 where u32: Copy {{", f"fn zi{k}_o(mut x: u32) -> u32 {{"]
    else:
        head = hhead
        edits = [f"fn zi{k}_h(x: u32) -> u64 {{", f"fn zi{k}_h(x: u64) -> u32 {{", f"const fn zi{k}_h(x: u32) -> u32 {{",
                 f"fn zi{k}_h(x: u32) -> u32 where u32: Copy {{", f"unsafe fn zi{k}_h(x: u32) -> u32 {{",
                 f"fn zi{k}_h(x: u32, _y: u8) -> u32 {{", f"fn zi{k}_h(x: u32) -> u32 where String: Copy {{"]
    drops = [(derive, derive.replace(t, "")) for t in ("Copy, ", ", Debug", "Clone, Copy, ")]
    return _header_walk(rng, fs, items, head, edits, drops)


def sig_states(rng, fs):
    """One op: (op name, states to visit). No state when it has no site.
    SIGOPS_ONLY=a,b limits the ops (a sweep that targets one path)."""
    only = [o for o in os.environ.get("SIGOPS_ONLY", "").split(",") if o in OPS]
    op = rng.choice(only or OPS)
    return op, globals()[op](rng, fs)


class External:
    """rustc 1.99 and baseline lust verdicts for op states (see the top)."""

    def __init__(self, rustc, baseline):
        self.rustc = rustc
        self.baseline = baseline if baseline and os.access(baseline, os.X_OK) else None
        if baseline and not self.baseline:
            print(f"sigops: baseline {baseline} is not an executable file; judging without it", flush=True)
        self.counts = {"states": 0, "false_accept": 0, "false_reject": 0, "known_fa": 0, "known_fr": 0,
                       "rustc_timeout": 0}
        self.tmp = tempfile.mkdtemp(prefix="lust-sigops-")

    def on(self):
        return self.rustc

    def _rustc_ok(self, root):
        try:
            # --emit=obj, not metadata: metadata skips the MIR deny lints (unconditional_panic,
            # arithmetic_overflow) and post-monomorphization errors that a real build rejects.
            r = subprocess.run(["rustc", "--edition", "2024", "--crate-type", "bin", "-C", "panic=abort",
                                "-C", "overflow-checks=on", "--emit=obj", "-o", os.path.join(self.tmp, "x.o"), root],
                               cwd=os.path.dirname(root), capture_output=True, text=True, timeout=60)
        except subprocess.TimeoutExpired:
            return None
        return r.returncode == 0

    def _base_ok(self, root, env):
        if not self.baseline:
            return None
        try:
            r = subprocess.run([self.baseline, "check", root], cwd=os.path.dirname(root), env=env,
                               capture_output=True, text=True, timeout=120)
        except subprocess.TimeoutExpired:
            return "hang"
        return r.returncode == 0

    def judge(self, root, warm_ok, cold_env):
        """None when the warm verdict stands; else why it fails."""
        if not self.rustc:
            return None
        self.counts["states"] += 1
        r = self._rustc_ok(root)
        if r is None:
            self.counts["rustc_timeout"] += 1
            return None
        if warm_ok == r:
            return None
        kind = "false_accept" if warm_ok else "false_reject"
        b = self._base_ok(root, cold_env)
        if b == warm_ok:
            self.counts["known_" + ("fa" if warm_ok else "fr")] += 1
            # SIGOPS_SAVE=<dir> keeps each known false accept's crate (a
            # rule-zero bug the baseline has too: report it).
            save = os.environ.get("SIGOPS_SAVE")
            if warm_ok and save:
                import shutil
                src = os.path.dirname(root)
                keep = lambda d, names: [n for n in names if not os.path.isdir(os.path.join(d, n)) and not n.endswith((".rs", ".toml"))]
                shutil.copytree(src, os.path.join(save, f"known_fa-{self.counts['known_fa']}"), ignore=keep, dirs_exist_ok=True)
            return None
        if b is None and not warm_ok:
            # No baseline: a false reject may be a known gap.
            self.counts["known_fr"] += 1
            return None
        self.counts[kind] += 1
        return f"new {kind} (rustc {'accepts' if r else 'rejects'}, baseline {b})"

    def summary(self):
        return "sigops: " + " ".join(f"{k}={v}" for k, v in self.counts.items())
