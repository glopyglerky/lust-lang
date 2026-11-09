//! Pass 2: one `syn` visitor per file. Every feature hit goes to
//! `Stats::add` under a dotted key (`item.fn`, `method.map`, `lt.ref.named`).

use crate::features::FeatureTable;
use crate::global::{Global, STD_ROOTS};
use crate::stats::Finding;
use crate::stats::{FnInfo, SigFacts, Stats};
use crate::walk::{Cat, File};
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::{HashMap, HashSet};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// True for `#[cfg(test)]` and `#[cfg(all(test, ..))]`.
pub fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    fn pred(m: &syn::Meta) -> bool {
        match m {
            syn::Meta::Path(p) => p.is_ident("test"),
            syn::Meta::List(l) if l.path.is_ident("all") => {
                l.parse_args_with(Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated).is_ok_and(|v| v.iter().any(pred))
            }
            _ => false,
        }
    }
    attrs.iter().any(|a| a.path().is_ident("cfg") && a.parse_args::<syn::Meta>().is_ok_and(|m| pred(&m)))
}

fn is_test_fn(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("test")) || is_cfg_test(attrs)
}

/// Flattens a `use` tree into (local name, full path) pairs. A glob gives
/// the name `*`.
pub fn flatten_use(t: &syn::UseTree, prefix: String, out: &mut Vec<(String, String)>) {
    match t {
        syn::UseTree::Path(p) => flatten_use(&p.tree, format!("{prefix}{}::", p.ident), out),
        syn::UseTree::Name(n) => {
            if n.ident == "self" {
                let full = prefix.trim_end_matches("::").to_string();
                let last = full.rsplit("::").next().unwrap_or("").to_string();
                out.push((last, full));
            } else {
                out.push((n.ident.to_string(), format!("{prefix}{}", n.ident)));
            }
        }
        syn::UseTree::Rename(r) => out.push((r.rename.to_string(), format!("{prefix}{}", r.ident))),
        syn::UseTree::Glob(_) => out.push(("*".into(), prefix.trim_end_matches("::").to_string())),
        syn::UseTree::Group(g) => {
            for i in &g.items {
                flatten_use(i, prefix.clone(), out);
            }
        }
    }
}

fn std_prelude(name: &str) -> Option<&'static str> {
    Some(match name {
        "Box" => "std::boxed::Box",
        "Vec" => "std::vec::Vec",
        "String" => "std::string::String",
        "ToString" => "std::string::ToString",
        "Option" => "std::option::Option",
        "Some" => "std::option::Option::Some",
        "None" => "std::option::Option::None",
        "Result" => "std::result::Result",
        "Ok" => "std::result::Result::Ok",
        "Err" => "std::result::Result::Err",
        "ToOwned" => "std::borrow::ToOwned",
        "Clone" => "std::clone::Clone",
        "Copy" => "std::marker::Copy",
        "Send" => "std::marker::Send",
        "Sync" => "std::marker::Sync",
        "Sized" => "std::marker::Sized",
        "Unpin" => "std::marker::Unpin",
        "Default" => "std::default::Default",
        "Iterator" => "std::iter::Iterator",
        "IntoIterator" => "std::iter::IntoIterator",
        "DoubleEndedIterator" => "std::iter::DoubleEndedIterator",
        "ExactSizeIterator" => "std::iter::ExactSizeIterator",
        "Extend" => "std::iter::Extend",
        "FromIterator" => "std::iter::FromIterator",
        "From" => "std::convert::From",
        "Into" => "std::convert::Into",
        "TryFrom" => "std::convert::TryFrom",
        "TryInto" => "std::convert::TryInto",
        "AsRef" => "std::convert::AsRef",
        "AsMut" => "std::convert::AsMut",
        "PartialEq" => "std::cmp::PartialEq",
        "Eq" => "std::cmp::Eq",
        "PartialOrd" => "std::cmp::PartialOrd",
        "Ord" => "std::cmp::Ord",
        "Drop" => "std::ops::Drop",
        "Fn" => "std::ops::Fn",
        "FnMut" => "std::ops::FnMut",
        "FnOnce" => "std::ops::FnOnce",
        "drop" => "std::mem::drop",
        _ => return None,
    })
}

const PRIMS: &[&str] =
    &["i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32", "f64", "bool", "char", "str"];

pub const ITER_SOURCES: &[&str] = &[
    "iter",
    "iter_mut",
    "into_iter",
    "chars",
    "char_indices",
    "bytes",
    "lines",
    "split",
    "rsplit",
    "splitn",
    "rsplitn",
    "split_terminator",
    "split_whitespace",
    "split_ascii_whitespace",
    "keys",
    "values",
    "values_mut",
    "drain",
    "windows",
    "chunks",
    "chunks_exact",
    "match_indices",
    "into_keys",
    "into_values",
    "encode_utf16",
];
pub const ITER_ADAPTERS: &[&str] = &[
    "map",
    "filter",
    "filter_map",
    "flat_map",
    "flatten",
    "enumerate",
    "zip",
    "chain",
    "rev",
    "skip",
    "take",
    "take_while",
    "skip_while",
    "map_while",
    "peekable",
    "cloned",
    "copied",
    "step_by",
    "scan",
    "inspect",
    "fuse",
    "cycle",
    "by_ref",
];
pub const ITER_CONSUMERS: &[&str] = &[
    "collect",
    "fold",
    "sum",
    "product",
    "count",
    "any",
    "all",
    "find",
    "find_map",
    "position",
    "rposition",
    "max",
    "min",
    "max_by",
    "min_by",
    "max_by_key",
    "min_by_key",
    "last",
    "nth",
    "for_each",
    "try_for_each",
    "try_fold",
    "partition",
    "unzip",
    "reduce",
    "next",
    "next_back",
    "rfind",
];

#[derive(Clone, Copy, PartialEq)]
enum Pos {
    Other,
    Arg,
    Ret,
    Field,
}

struct V<'a> {
    file: u32,
    rel: &'a str,
    text: &'a str,
    feats: &'a FeatureTable,
    target_inactive: bool,
    g: &'a Global,
    s: &'a mut Stats,
    cats: Vec<Cat>,
    fns: Vec<u32>,
    impl_self: Vec<String>,
    aliases: HashMap<String, String>,
    tparams: Vec<HashSet<String>>,
    pos: Pos,
    receivers: HashSet<usize>,
    /// Inside `impl Trait for T`.
    in_trait_impl: bool,
}

pub fn analyze_file(idx: u32, f: &File, g: &Global, feats: &FeatureTable, s: &mut Stats) {
    s.lines_by_cat[f.cat as usize] += f.lines;
    s.files_by_cat[f.cat as usize] += 1;
    let Some(ast) = &f.ast else { return };
    let mut aliases = HashMap::new();
    // Local items shadow std prelude names (a local `type Result<T>`).
    let mut uses = Vec::new();
    collect_local(&ast.items, &mut aliases, &mut uses);
    let mut has_prelude_glob = false;
    for (name, path) in uses {
        if name == "*" {
            if path.ends_with("prelude") && !path.starts_with("std") {
                has_prelude_glob = true;
            }
        } else if name != "_" {
            aliases.insert(name, path);
        }
    }
    if has_prelude_glob {
        for (k, v) in &g.prelude {
            aliases.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    let mut v = V {
        file: idx,
        rel: &f.rel,
        text: &f.text,
        feats,
        target_inactive: f.cfg_state == crate::cfg_eval::Tri::False,
        g,
        s,
        cats: vec![f.cat],
        fns: Vec::new(),
        impl_self: Vec::new(),
        aliases,
        tparams: Vec::new(),
        pos: Pos::Other,
        receivers: HashSet::new(),
        in_trait_impl: false,
    };
    v.visit_file(ast);
}

fn collect_local(items: &[syn::Item], aliases: &mut HashMap<String, String>, uses: &mut Vec<(String, String)>) {
    for it in items {
        let name = match it {
            syn::Item::Struct(i) => Some(&i.ident),
            syn::Item::Enum(i) => Some(&i.ident),
            syn::Item::Type(i) => Some(&i.ident),
            syn::Item::Trait(i) => Some(&i.ident),
            syn::Item::Union(i) => Some(&i.ident),
            syn::Item::Fn(i) => Some(&i.sig.ident),
            syn::Item::Const(i) => Some(&i.ident),
            syn::Item::Static(i) => Some(&i.ident),
            syn::Item::Use(u) => {
                flatten_use(&u.tree, String::new(), uses);
                None
            }
            syn::Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    collect_local(inner, aliases, uses);
                }
                None
            }
            _ => None,
        };
        if let Some(n) = name {
            aliases.insert(n.to_string(), format!("crate::local::{n}"));
        }
    }
}

fn last_ident(p: &syn::Path) -> String {
    p.segments.last().map(|s| s.ident.to_string()).unwrap_or_default()
}

fn tok(t: &impl ToTokens) -> String {
    t.to_token_stream().to_string().replace(' ', "")
}

/// Facts about a list of types in a signature.
#[derive(Default)]
struct TypeScan<'g> {
    g: Option<&'g Global>,
    refs: usize,
    named: Vec<String>,
    anon: usize,
    stat: usize,
    lt_types: Vec<String>,
}
impl<'ast> Visit<'ast> for TypeScan<'_> {
    fn visit_type_reference(&mut self, t: &'ast syn::TypeReference) {
        self.refs += 1;
        visit::visit_type_reference(self, t);
    }
    fn visit_lifetime(&mut self, l: &'ast syn::Lifetime) {
        match l.ident.to_string().as_str() {
            "static" => self.stat += 1,
            "_" => self.anon += 1,
            s => self.named.push(s.to_string()),
        }
    }
    fn visit_type_path(&mut self, t: &'ast syn::TypePath) {
        let n = last_ident(&t.path);
        if self.g.is_some_and(|g| g.lt_types.contains_key(&n)) {
            self.lt_types.push(n);
        }
        visit::visit_type_path(self, t);
    }
}

impl<'a> V<'a> {
    fn cat(&self) -> Cat {
        *self.cats.last().unwrap()
    }

    /// Counts the lines of a test item inside a non-test file.
    fn inline_test(&mut self, span: Span) {
        if self.cat() != Cat::Test {
            let n = (span.end().line - span.start().line + 1) as u64;
            let cat = self.cat();
            self.s.inline_test_lines[cat as usize] += n as usize;
        }
    }

    fn hit(&mut self, key: impl AsRef<str>, span: Span) {
        self.hit_n(key, 1, span);
    }

    fn hit_n(&mut self, key: impl AsRef<str>, n: u64, span: Span) {
        let key = key.as_ref();
        let cat = self.cat();
        let f = self.fns.last().copied();
        let rel = self.rel;
        self.s.add_n(key, n, cat, self.file, f, || format!("{}:{}", rel, span.start().line));
        if n != 1 {
            return;
        }
        let Some(feat) = self.feats.classify(key) else { return };
        if !feat.is_gap() || !FeatureTable::is_primary(feat, key) {
            return;
        }
        let range = span.byte_range();
        let (start, end) = if range.start <= range.end && range.end <= self.text.len() { (range.start, range.end) } else { (0, 0) };
        if !self.s.seen_findings.insert((feat.id.clone(), self.file, start, end)) {
            return;
        }
        let scope: &'static str = if cat == Cat::Test {
            "test-only"
        } else if self.target_inactive {
            "target-specific"
        } else {
            "active"
        };
        self.s.findings.push(Finding {
            feature: feat.id.clone(),
            raw_key: key.to_string(),
            file_idx: self.file,
            byte_start: start,
            byte_end: end,
            line: span.start().line,
            column: span.start().column + 1,
            snippet: snippet_of(self.text, start, end),
            scope,
        });
    }

    /// Full path for std and external-crate paths; `None` for user paths.
    fn resolve(&self, p: &syn::Path) -> Option<String> {
        let segs: Vec<String> = p.segments.iter().map(|s| s.ident.to_string()).collect();
        let first = segs.first()?.as_str();
        // A bare one-segment path with no leading `::` names a local
        // value, function, or imported name, never a crate root: a crate
        // name alone is not a value or a type. Without this guard a
        // local `notify` or `same_file` reads as the crate of that name.
        let rooted = p.leading_colon.is_some() || segs.len() > 1;
        let base = if rooted && (STD_ROOTS.contains(&first) || self.g.ext_crates.contains(first)) {
            segs.join("::")
        } else if let Some(full) = self.aliases.get(first) {
            std::iter::once(full.clone()).chain(segs[1..].iter().cloned()).collect::<Vec<_>>().join("::")
        } else if let Some(full) = std_prelude(first) {
            std::iter::once(full.to_string()).chain(segs[1..].iter().cloned()).collect::<Vec<_>>().join("::")
        } else {
            return None;
        };
        let root = base.split("::").next().unwrap_or("");
        if STD_ROOTS.contains(&root) {
            let rest = &base[root.len()..];
            Some(format!("std{rest}"))
        } else if self.g.ext_crates.contains(root) {
            Some(base)
        } else {
            None
        }
    }

    fn path_key(&self, p: &syn::Path) -> String {
        match self.resolve(p) {
            Some(r) => r,
            None => format!("user:{}", last_ident(p)),
        }
    }

    fn in_tparams(&self, name: &str) -> bool {
        self.tparams.iter().any(|s| s.contains(name))
    }

    fn generics(&mut self, g: &syn::Generics, kind: &str) {
        let sp = g.span();
        for p in &g.params {
            match p {
                syn::GenericParam::Lifetime(l) => {
                    self.hit(format!("gen.lifetime_param.{kind}"), sp);
                    if !l.bounds.is_empty() {
                        self.hit("bound.lifetime_outlives", sp);
                    }
                }
                syn::GenericParam::Type(t) => {
                    self.hit(format!("gen.type_param.{kind}"), sp);
                    if t.default.is_some() {
                        self.hit("gen.type_param_default", sp);
                    }
                    for b in &t.bounds {
                        self.bound(b, sp);
                    }
                }
                syn::GenericParam::Const(_) => self.hit(format!("gen.const_param.{kind}"), sp),
            }
        }
        if let Some(w) = &g.where_clause {
            self.hit(format!("gen.where.{kind}"), sp);
            for pred in &w.predicates {
                match pred {
                    syn::WherePredicate::Type(t) => {
                        if t.lifetimes.is_some() {
                            self.hit("lt.hrtb.where", sp);
                        }
                        if !matches!(&t.bounded_ty, syn::Type::Path(tp) if tp.path.segments.len() == 1 && tp.qself.is_none()) {
                            self.hit("gen.where.complex_lhs", sp);
                        }
                        for b in &t.bounds {
                            self.bound(b, sp);
                        }
                    }
                    syn::WherePredicate::Lifetime(_) => self.hit("bound.lifetime_outlives", sp),
                    _ => {}
                }
            }
        }
    }

    fn bound(&mut self, b: &syn::TypeParamBound, sp: Span) {
        match b {
            syn::TypeParamBound::Trait(t) => {
                if matches!(t.modifier, syn::TraitBoundModifier::Maybe(_)) {
                    self.hit("bound.maybe_sized", sp);
                    return;
                }
                if t.lifetimes.is_some() {
                    self.hit(if sugar_binder_canonical(t) { "lt.hrtb.bound" } else { "lt.hrtb.noncanon" }, sp);
                }
                let k = self.path_key(&t.path);
                self.hit(format!("bound.{k}"), sp);
                if let Some(last) = t.path.segments.last() {
                    if matches!(last.arguments, syn::PathArguments::AngleBracketed(ref a) if a.args.iter().any(|x| matches!(x, syn::GenericArgument::AssocType(_))))
                    {
                        self.hit("bound.assoc_type_eq", sp);
                    }
                }
            }
            syn::TypeParamBound::Lifetime(l) => {
                if l.ident == "static" {
                    self.hit("bound.static", sp);
                } else {
                    self.hit("bound.lifetime", sp);
                }
            }
            _ => self.hit("bound.other", sp),
        }
    }

    fn attrs_derive(&mut self, attrs: &[syn::Attribute]) {
        for a in attrs {
            if a.path().is_ident("derive") {
                if let Ok(list) = a.parse_args_with(Punctuated::<syn::Path, syn::Token![,]>::parse_terminated) {
                    for p in list {
                        self.hit(format!("derive.{}", last_ident(&p)), a.span());
                    }
                }
            }
        }
    }

    fn type_params_of(g: &syn::Generics) -> HashSet<String> {
        g.type_params().map(|t| t.ident.to_string()).collect()
    }

    /// Records a function and its signature facts; returns the fn id.
    fn begin_fn(&mut self, sig: &syn::Signature, kind: &str) -> u32 {
        let sp = sig.ident.span();
        let id = self.s.fns.len() as u32;
        let mut sf = SigFacts::default();
        let mut ins = TypeScan { g: Some(self.g), ..Default::default() };
        let mut self_lt: Option<Option<String>> = None;
        for a in &sig.inputs {
            match a {
                syn::FnArg::Receiver(r) => {
                    sf.is_method = true;
                    if let Some((_, lt)) = &r.reference {
                        sf.self_ref = true;
                        sf.self_mut = r.mutability.is_some();
                        self_lt = Some(lt.as_ref().map(|l| l.ident.to_string()));
                        if let Some(l) = lt {
                            ins.visit_lifetime(l);
                        }
                    } else if r.colon_token.is_some() {
                        // `self: Rc<Self>`
                        ins.visit_type(&r.ty);
                    }
                }
                syn::FnArg::Typed(t) => ins.visit_type(&t.ty),
            }
        }
        let mut out = TypeScan { g: Some(self.g), ..Default::default() };
        if let syn::ReturnType::Type(_, t) = &sig.output {
            out.visit_type(t);
        }
        sf.explicit_lt_params = sig.generics.lifetimes().count();
        sf.type_params = sig.generics.type_params().count();
        sf.ref_in_args = ins.refs > 0 || sf.self_ref;
        sf.ref_in_ret = out.refs > 0;
        sf.explicit_lt_use = !ins.named.is_empty() || !out.named.is_empty();
        sf.anon_lt_use = ins.anon + out.anon > 0;
        sf.static_lt_use = ins.stat + out.stat > 0;
        sf.lt_types = ins.lt_types.iter().chain(out.lt_types.iter()).cloned().collect();
        sf.lt_types.sort();
        sf.lt_types.dedup();

        self.fns.push(id);
        self.hit(format!("fn.kind.{kind}"), sp);
        if sf.explicit_lt_params > 0 {
            self.hit("sig.lifetime_params", sp);
        }
        if sf.ref_in_args {
            self.hit("sig.ref_in_args", sp);
        }
        if sf.ref_in_ret {
            self.hit("sig.ref_in_ret", sp);
        }
        if sf.explicit_lt_use {
            self.hit("sig.named_lifetime_use", sp);
        }
        if sf.anon_lt_use {
            self.hit("sig.anon_lifetime_use", sp);
        }
        if sf.static_lt_use {
            self.hit("sig.static_lifetime_use", sp);
        }
        if !sf.lt_types.is_empty() {
            self.hit("sig.mentions_lt_type", sp);
        }
        if out.stat > 0 {
            self.hit("sig.ret_static", sp);
        }
        // Can Rust's elision rules give the named return lifetime?
        let mut ret_named: Vec<&String> = out.named.iter().collect();
        ret_named.sort();
        ret_named.dedup();
        if !ret_named.is_empty() {
            let elidable = match &self_lt {
                Some(Some(sl)) => ret_named.len() == 1 && ret_named[0] == sl,
                Some(None) => false,
                None => {
                    let positions = ins.refs + ins.named.len() + ins.anon;
                    let distinct: HashSet<&String> = ins.named.iter().collect();
                    positions >= 1 && distinct.len() == 1 && ret_named.len() == 1 && distinct.contains(ret_named[0]) && ins.refs <= 1
                }
            };
            self.hit(if elidable { "sig.ret_named_elidable" } else { "sig.ret_named_needs_name" }, sp);
        } else if out.refs > 0 || out.anon > 0 {
            self.hit("sig.ret_elided", sp);
        }
        let name = match self.impl_self.last() {
            Some(t) => format!("{t}::{}", sig.ident),
            None => sig.ident.to_string(),
        };
        self.s.fns.push(FnInfo { file: self.file, cat: self.cat(), name, line: sp.start().line, sig: sf });
        id
    }

    fn signature_types(&mut self, sig: &syn::Signature) {
        let old = self.pos;
        for a in &sig.inputs {
            match a {
                syn::FnArg::Receiver(r) => {
                    let k = match (&r.reference, r.mutability.is_some(), r.colon_token.is_some()) {
                        (Some((_, Some(_))), false, _) => "ref_named",
                        (Some((_, Some(_))), true, _) => "mut_named",
                        (Some(_), false, _) => "ref",
                        (Some(_), true, _) => "mut",
                        (None, _, true) => "typed",
                        (None, true, false) => "value_mut",
                        (None, false, false) => "value",
                    };
                    self.hit(format!("fn.receiver.{k}"), r.self_token.span);
                    if r.colon_token.is_some() {
                        self.hit(format!("fn.receiver.typed.{}", tok(&*r.ty)), r.self_token.span);
                    }
                }
                syn::FnArg::Typed(t) => {
                    self.pos = Pos::Arg;
                    self.visit_type(&t.ty);
                    self.pos = Pos::Other;
                    self.visit_pat(&t.pat);
                }
            }
        }
        if let syn::ReturnType::Type(_, t) = &sig.output {
            self.pos = Pos::Ret;
            self.visit_type(t);
        }
        self.pos = old;
        if sig.asyncness.is_some() {
            self.hit("fn.async", sig.ident.span());
        }
        if sig.constness.is_some() {
            self.hit("fn.const", sig.ident.span());
        }
        if sig.unsafety.is_some() {
            self.hit("fn.unsafe", sig.ident.span());
        }
        if sig.abi.is_some() {
            self.hit("fn.extern_abi", sig.ident.span());
        }
        if sig.variadic.is_some() {
            self.hit("fn.variadic", sig.ident.span());
        }
    }

    fn fields(&mut self, fields: &syn::Fields) {
        for f in fields {
            let sp = f.ty.span();
            let mut ts = TypeScan { g: Some(self.g), ..Default::default() };
            ts.visit_type(&f.ty);
            match &f.ty {
                syn::Type::Reference(r) => {
                    let st = r.lifetime.as_ref().is_some_and(|l| l.ident == "static");
                    self.hit(if st { "lt.field.ref_static" } else { "lt.field.ref" }, sp);
                    if !st {
                        self.hit(format!("lt.field.ref_to.{}", short_type(&r.elem)), sp);
                    }
                }
                _ if ts.refs > 0 => self.hit("lt.field.ref_nested", sp),
                _ => {}
            }
            if !ts.named.is_empty() {
                self.hit("lt.field.named_lifetime", sp);
            }
            if ts.stat > 0 {
                self.hit("lt.field.static_lifetime", sp);
            }
            if !ts.lt_types.is_empty() {
                self.hit("lt.field.lt_type", sp);
            }
            let s = tok(&f.ty);
            if s.contains("dynFn") || s.starts_with("fn(") || s.contains("<fn(") {
                self.hit("field.fn_value", sp);
            }
        }
    }

    fn method_chain(&mut self, m: &syn::ExprMethodCall) {
        let mut names = vec![m.method.to_string()];
        let mut cur = &*m.receiver;
        loop {
            match cur {
                syn::Expr::MethodCall(r) => {
                    names.push(r.method.to_string());
                    cur = &r.receiver;
                }
                syn::Expr::Try(t) => cur = &t.expr,
                _ => break,
            }
        }
        names.reverse();
        let sp = m.method.span();
        if names.len() >= 2 {
            self.hit(format!("chain.len.{}", names.len().min(8)), sp);
        }
        let src = names.iter().position(|n| ITER_SOURCES.contains(&n.as_str()));
        let adapters = names.iter().filter(|n| ITER_ADAPTERS.contains(&n.as_str())).count();
        let consumer = names.iter().any(|n| ITER_CONSUMERS.contains(&n.as_str()));
        if src.is_some() && (adapters > 0 || consumer) || adapters > 0 {
            self.hit("iter.chain", sp);
            self.hit(format!("iter.chain.adapters.{}", adapters.min(6)), sp);
            if src.is_none() {
                self.hit("iter.chain.unknown_source", sp);
            }
        }
    }

    fn format_string(&mut self, lit: &syn::LitStr) {
        let s = lit.value();
        let b = s.as_bytes();
        let mut i = 0;
        while i < b.len() {
            if b[i] == b'{' {
                if i + 1 < b.len() && b[i + 1] == b'{' {
                    i += 2;
                    continue;
                }
                let Some(end) = s[i..].find('}') else { break };
                let inner = &s[i + 1..i + end];
                let (arg, spec) = inner.split_once(':').unwrap_or((inner, ""));
                let kind = match spec {
                    "" => "display",
                    "?" => "debug",
                    "#?" => "debug_pretty",
                    "x" | "X" | "#x" | "#X" | "b" | "#b" | "o" | "#o" | "e" | "E" => "radix_or_exp",
                    _ => "width_prec_align",
                };
                self.hit(format!("fmt.spec.{kind}"), lit.span());
                if !arg.is_empty() && !arg.chars().all(|c| c.is_ascii_digit()) {
                    self.hit("fmt.inline_capture", lit.span());
                }
                i += end + 1;
            } else {
                i += 1;
            }
        }
    }

    fn visit_exprs(&mut self, list: &Punctuated<syn::Expr, syn::Token![,]>) {
        for e in list {
            match e {
                syn::Expr::Assign(a) => self.visit_expr(&a.right),
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => self.format_string(s),
                _ => self.visit_expr(e),
            }
        }
    }
}

/// One-line source text for a byte range, capped at 120 chars.
fn snippet_of(text: &str, start: usize, end: usize) -> String {
    let Some(raw) = text.get(start..end) else { return String::new() };
    let one: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if one.chars().count() > 120 {
        let cut: String = one.chars().take(117).collect();
        format!("{cut}...")
    } else {
        one
    }
}

/// True when a type mentions a lifetime: a ref lifetime or a lifetime
/// generic argument, at any depth.
/// The lifetime names a `for<..>` binder declares.
fn binder_names(b: Option<&syn::BoundLifetimes>) -> Vec<String> {
    b.map(|b| {
        b.lifetimes
            .iter()
            .filter_map(|p| if let syn::GenericParam::Lifetime(l) = p { Some(l.lifetime.ident.to_string()) } else { None })
            .collect()
    })
    .unwrap_or_default()
}

/// A `for<..>` binder on `Fn*` sugar that lust reads as elision.
fn sugar_binder_canonical(tb: &syn::TraitBound) -> bool {
    let binder = binder_names(tb.lifetimes.as_ref());
    match tb.path.segments.last().map(|s| &s.arguments) {
        Some(syn::PathArguments::Parenthesized(p)) => {
            let ins: Vec<&syn::Type> = p.inputs.iter().collect();
            let out = match &p.output {
                syn::ReturnType::Type(_, t) => Some(&**t),
                syn::ReturnType::Default => None,
            };
            binder_canonical(&binder, &ins, out)
        }
        _ => false,
    }
}

/// True when a binder's lifetimes say what elision says (lust's
/// parser/hrtb.rs): each at most once in the inputs, in plain places
/// (references, lifetime args, through type args), and in the output
/// only when it is the one input lifetime.
fn binder_canonical(binder: &[String], ins: &[&syn::Type], out: Option<&syn::Type>) -> bool {
    let mut si = LtScan::default();
    for t in ins {
        lt_scan(t, binder, true, &mut si);
    }
    let mut so = LtScan::default();
    if let Some(t) = out {
        lt_scan(t, binder, true, &mut so);
    }
    if si.complex_binder || so.complex_binder {
        return false;
    }
    let mut seen = HashSet::new();
    if !si.binder_uses.iter().all(|b| seen.insert(b.clone())) {
        return false;
    }
    so.binder_uses.iter().all(|b| !si.opaque && si.lts == 1 && si.binder_uses.first() == Some(b))
}

/// What `lt_scan` finds in a fn pointer signature.
#[derive(Default)]
struct LtScan {
    /// Named lifetimes of the enclosing item (not `'static`, `'_`, or a binder's).
    free: HashSet<String>,
    /// A form lust cuts: a named lifetime inside a nested fn pointer,
    /// `dyn`, or `Fn` type, or a `'static` that is not on a reference.
    complex: bool,
    /// Binder lifetimes in plain places, in order.
    binder_uses: Vec<String>,
    /// A binder lifetime inside a nested type.
    complex_binder: bool,
    /// Lifetimes elision counts (references and lifetime args).
    lts: u32,
    /// A part elision does not see into the same way.
    opaque: bool,
}

/// Scans a type of a fn pointer signature for its lifetimes (`LtScan`).
fn lt_scan(t: &syn::Type, binder: &[String], plain: bool, s: &mut LtScan) {
    let mut note = |l: &syn::Lifetime, on_ref: bool, s: &mut LtScan| {
        let n = l.ident.to_string();
        if n == "_" {
            return;
        }
        if binder.contains(&n) {
            if plain {
                s.binder_uses.push(n);
            } else {
                s.complex_binder = true;
            }
        } else if n == "static" {
            if !on_ref {
                s.complex = true;
            }
        } else if plain {
            s.free.insert(n);
        } else {
            s.complex = true;
        }
    };
    match t {
        syn::Type::Reference(r) => {
            if plain {
                s.lts += 1;
            }
            if let Some(l) = &r.lifetime {
                note(l, true, s);
            }
            lt_scan(&r.elem, binder, plain, s);
        }
        syn::Type::Path(p) => {
            for seg in &p.path.segments {
                match &seg.arguments {
                    syn::PathArguments::AngleBracketed(a) => {
                        for arg in &a.args {
                            match arg {
                                syn::GenericArgument::Lifetime(l) => {
                                    if plain {
                                        s.lts += 1;
                                    }
                                    note(l, false, s);
                                }
                                syn::GenericArgument::Type(inner) => lt_scan(inner, binder, plain, s),
                                syn::GenericArgument::AssocType(b) => {
                                    s.opaque = true;
                                    lt_scan(&b.ty, binder, false, s);
                                }
                                _ => {}
                            }
                        }
                    }
                    syn::PathArguments::Parenthesized(par) => {
                        s.opaque = true;
                        for i in &par.inputs {
                            lt_scan(i, binder, false, s);
                        }
                        if let syn::ReturnType::Type(_, o) = &par.output {
                            lt_scan(o, binder, false, s);
                        }
                    }
                    syn::PathArguments::None => {}
                }
            }
        }
        syn::Type::Tuple(tt) => tt.elems.iter().for_each(|e| lt_scan(e, binder, plain, s)),
        syn::Type::Slice(x) => lt_scan(&x.elem, binder, plain, s),
        syn::Type::Array(x) => lt_scan(&x.elem, binder, plain, s),
        syn::Type::Ptr(x) => lt_scan(&x.elem, binder, plain, s),
        syn::Type::Paren(x) => lt_scan(&x.elem, binder, plain, s),
        syn::Type::Group(x) => lt_scan(&x.elem, binder, plain, s),
        syn::Type::BareFn(f) => {
            s.opaque = true;
            for a in &f.inputs {
                lt_scan(&a.ty, binder, false, s);
            }
            if let syn::ReturnType::Type(_, o) = &f.output {
                lt_scan(o, binder, false, s);
            }
        }
        syn::Type::TraitObject(o) => {
            s.opaque = true;
            dyn_bounds_scan(o.bounds.iter(), binder, s, &mut note);
        }
        syn::Type::ImplTrait(o) => {
            s.opaque = true;
            dyn_bounds_scan(o.bounds.iter(), binder, s, &mut note);
        }
        _ => {}
    }
}

/// `lt_scan` over the bounds of a `dyn` or `impl` type (never plain).
fn dyn_bounds_scan<'b>(
    bounds: impl Iterator<Item = &'b syn::TypeParamBound>,
    binder: &[String],
    s: &mut LtScan,
    note: &mut impl FnMut(&syn::Lifetime, bool, &mut LtScan),
) {
    for b in bounds {
        match b {
            syn::TypeParamBound::Lifetime(l) => note(l, false, s),
            syn::TypeParamBound::Trait(tb) => {
                let ty = syn::Type::Path(syn::TypePath { qself: None, path: tb.path.clone() });
                lt_scan(&ty, binder, false, s);
            }
            _ => {}
        }
    }
}


fn short_type(t: &syn::Type) -> String {
    match t {
        syn::Type::Path(p) => last_ident(&p.path),
        syn::Type::Slice(_) => "slice".into(),
        syn::Type::TraitObject(_) => "dyn".into(),
        syn::Type::Reference(_) => "ref".into(),
        _ => "other".into(),
    }
}

const WRAPPERS: &[&str] = &[
    "Box", "Rc", "Arc", "Weak", "Cell", "RefCell", "Mutex", "RwLock", "Option", "Vec", "OnceCell", "OnceLock", "Cow", "LazyLock",
    "LazyCell",
];

impl<'ast> Visit<'ast> for V<'_> {
    // ---------- items ----------
    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
        let test = is_test_fn(&i.attrs);
        if test {
            self.inline_test(i.span());
            self.cats.push(Cat::Test);
        }
        let nested = !self.fns.is_empty();
        let kind = if nested { "nested" } else { "free" };
        self.begin_fn(&i.sig, kind);
        self.hit("item.fn", i.sig.ident.span());
        if i.sig.generics.type_params().count() > 0 {
            self.hit("fn.generic", i.sig.ident.span());
        }
        self.generics(&i.sig.generics, "fn");
        self.tparams.push(Self::type_params_of(&i.sig.generics));
        for a in &i.attrs {
            self.visit_attribute(a);
        }
        self.signature_types(&i.sig);
        self.visit_block(&i.block);
        self.tparams.pop();
        self.fns.pop();
        if test {
            self.cats.pop();
        }
    }

    fn visit_impl_item_fn(&mut self, i: &'ast syn::ImplItemFn) {
        let test = is_test_fn(&i.attrs);
        if test {
            self.inline_test(i.span());
            self.cats.push(Cat::Test);
        }
        let kind = if self.in_trait_impl { "trait_impl" } else { "impl" };
        self.begin_fn(&i.sig, kind);
        if i.sig.generics.type_params().count() > 0 {
            self.hit("fn.generic_method", i.sig.ident.span());
        }
        self.generics(&i.sig.generics, "method");
        self.tparams.push(Self::type_params_of(&i.sig.generics));
        for a in &i.attrs {
            self.visit_attribute(a);
        }
        self.signature_types(&i.sig);
        self.visit_block(&i.block);
        self.tparams.pop();
        self.fns.pop();
        if test {
            self.cats.pop();
        }
    }

    fn visit_trait_item_fn(&mut self, i: &'ast syn::TraitItemFn) {
        let sp = i.sig.ident.span();
        self.generics(&i.sig.generics, "trait_method");
        if i.sig.generics.type_params().count() > 0 {
            self.hit("trait.generic_method", sp);
        }
        match &i.default {
            Some(b) => {
                self.hit("trait.default_method", sp);
                self.begin_fn(&i.sig, "trait_default");
                self.tparams.push(Self::type_params_of(&i.sig.generics));
                self.signature_types(&i.sig);
                self.visit_block(b);
                self.tparams.pop();
                self.fns.pop();
            }
            None => {
                self.hit("trait.required_method", sp);
                self.signature_types(&i.sig);
            }
        }
        if let syn::ReturnType::Type(_, t) = &i.sig.output {
            if matches!(**t, syn::Type::ImplTrait(_)) {
                self.hit("trait.rpitit", sp);
            }
        }
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let test = is_cfg_test(&i.attrs);
        if test {
            self.inline_test(i.span());
            self.cats.push(Cat::Test);
        }
        let sp = i.impl_token.span;
        let self_name = match &*i.self_ty {
            syn::Type::Path(p) => last_ident(&p.path),
            other => tok(other),
        };
        let tps = Self::type_params_of(&i.generics);
        if i.generics.lifetimes().count() > 0 {
            self.hit("impl.lifetime_params", sp);
        }
        if !tps.is_empty() {
            self.hit("impl.generic", sp);
        }
        if i.unsafety.is_some() {
            self.hit("impl.unsafe", sp);
        }
        match &i.trait_ {
            None => {
                self.hit("item.impl.inherent", sp);
                if self.g.user_types.contains(&self_name) || matches!(&*i.self_ty, syn::Type::Path(p) if self.resolve(&p.path).is_none()) {
                } else {
                    self.hit("impl.inherent_on_foreign", sp);
                }
            }
            Some((neg, path, _)) => {
                self.hit("item.impl.trait", sp);
                if neg.is_some() {
                    self.hit("impl.negative", sp);
                }
                let k = self.path_key(path);
                self.hit(format!("impl.trait.{k}"), sp);
                if k.starts_with("user:") {
                    self.hit("impl.user_trait", sp);
                }
                let blanket = match &*i.self_ty {
                    syn::Type::Path(p) => {
                        p.qself.is_none() && p.path.segments.len() == 1 && tps.contains(&p.path.segments[0].ident.to_string())
                    }
                    syn::Type::Reference(r) => {
                        matches!(&*r.elem, syn::Type::Path(p) if p.path.segments.len() == 1 && tps.contains(&p.path.segments[0].ident.to_string()))
                    }
                    _ => false,
                };
                if blanket {
                    self.hit("impl.blanket", sp);
                }
                if matches!(&*i.self_ty, syn::Type::Reference(_)) {
                    self.hit("impl.for_reference", sp);
                }
                if matches!(&*i.self_ty, syn::Type::TraitObject(_)) {
                    self.hit("impl.for_dyn", sp);
                }
                for it in &i.items {
                    if let syn::ImplItem::Type(t) = it {
                        self.hit("impl.assoc_type", t.ident.span());
                        if !t.generics.params.is_empty() {
                            self.hit("impl.gat", t.ident.span());
                        }
                    }
                }
            }
        }
        for it in &i.items {
            if let syn::ImplItem::Const(c) = it {
                self.hit("impl.assoc_const", c.ident.span());
            }
        }
        self.generics(&i.generics, "impl");
        self.impl_self.push(self_name);
        self.tparams.push(tps);
        for a in &i.attrs {
            self.visit_attribute(a);
        }
        if let Some((_, p, _)) = &i.trait_ {
            // Visit trait generic args as types (From<X>).
            for seg in &p.segments {
                self.visit_path_arguments(&seg.arguments);
            }
        }
        self.visit_type(&i.self_ty);
        let old = std::mem::replace(&mut self.in_trait_impl, i.trait_.is_some());
        for it in &i.items {
            self.visit_impl_item(it);
        }
        self.in_trait_impl = old;
        self.tparams.pop();
        self.impl_self.pop();
        if test {
            self.cats.pop();
        }
    }

    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let sp = i.ident.span();
        let k = match &i.fields {
            syn::Fields::Named(_) => "named",
            syn::Fields::Unnamed(_) => "tuple",
            syn::Fields::Unit => "unit",
        };
        self.hit(format!("item.struct.{k}"), sp);
        if let syn::Fields::Unnamed(u) = &i.fields {
            if u.unnamed.len() == 1 {
                if let syn::Type::Path(p) = &u.unnamed[0].ty {
                    let n = last_ident(&p.path);
                    if ["u32", "u64", "usize", "u16", "u8", "i32", "NonZeroU32", "NonZeroU64"].contains(&n.as_str()) {
                        self.hit("item.struct.int_newtype", sp);
                    }
                }
            }
        }
        if i.generics.type_params().count() > 0 {
            self.hit("item.struct.generic", sp);
        }
        if i.generics.lifetimes().count() > 0 {
            self.hit("item.struct.lifetime", sp);
        }
        self.generics(&i.generics, "struct");
        self.attrs_derive(&i.attrs);
        self.fields(&i.fields);
        self.hit_n("item.struct.fields", i.fields.len() as u64, sp);
        self.tparams.push(Self::type_params_of(&i.generics));
        let old = self.pos;
        self.pos = Pos::Field;
        visit::visit_item_struct(self, i);
        self.pos = old;
        self.tparams.pop();
    }

    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let sp = i.ident.span();
        self.hit("item.enum", sp);
        self.hit_n("enum.variants", i.variants.len() as u64, sp);
        let mut payload = false;
        for v in &i.variants {
            let k = match &v.fields {
                syn::Fields::Named(_) => "struct",
                syn::Fields::Unnamed(_) => "tuple",
                syn::Fields::Unit => "unit",
            };
            payload |= k != "unit";
            self.hit(format!("enum.variant.{k}"), v.ident.span());
            if v.discriminant.is_some() {
                self.hit("enum.discriminant", v.ident.span());
            }
            self.fields(&v.fields);
        }
        self.hit(if payload { "item.enum.with_payload" } else { "item.enum.fieldless" }, sp);
        if i.generics.type_params().count() > 0 {
            self.hit("item.enum.generic", sp);
        }
        if i.generics.lifetimes().count() > 0 {
            self.hit("item.enum.lifetime", sp);
        }
        self.generics(&i.generics, "enum");
        self.attrs_derive(&i.attrs);
        self.tparams.push(Self::type_params_of(&i.generics));
        let old = self.pos;
        self.pos = Pos::Field;
        visit::visit_item_enum(self, i);
        self.pos = old;
        self.tparams.pop();
    }

    fn visit_item_union(&mut self, i: &'ast syn::ItemUnion) {
        self.hit("item.union", i.ident.span());
        visit::visit_item_union(self, i);
    }

    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        let sp = i.ident.span();
        self.hit("item.trait", sp);
        if i.unsafety.is_some() {
            self.hit("trait.unsafe", sp);
        }
        if i.auto_token.is_some() {
            self.hit("trait.auto", sp);
        }
        if !i.supertraits.is_empty() {
            self.hit("trait.supertraits", sp);
            for b in &i.supertraits {
                self.bound(b, sp);
            }
        }
        if i.generics.type_params().count() > 0 {
            self.hit("trait.generic", sp);
        }
        if i.generics.lifetimes().count() > 0 {
            self.hit("trait.lifetime", sp);
        }
        self.generics(&i.generics, "trait");
        for it in &i.items {
            match it {
                syn::TraitItem::Type(t) => {
                    self.hit("trait.assoc_type", t.ident.span());
                    if !t.generics.params.is_empty() {
                        self.hit("trait.gat", t.ident.span());
                    }
                    if t.default.is_some() {
                        self.hit("trait.assoc_type_default", t.ident.span());
                    }
                }
                syn::TraitItem::Const(c) => self.hit("trait.assoc_const", c.ident.span()),
                _ => {}
            }
        }
        let mut tps = Self::type_params_of(&i.generics);
        tps.insert("Self".into());
        self.tparams.push(tps);
        self.impl_self.push(i.ident.to_string());
        visit::visit_item_trait(self, i);
        self.impl_self.pop();
        self.tparams.pop();
    }

    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        let sp = i.ident.span();
        self.hit("item.type_alias", sp);
        if !i.generics.params.is_empty() {
            self.hit("item.type_alias.generic", sp);
        }
        self.generics(&i.generics, "alias");
        visit::visit_item_type(self, i);
    }

    fn visit_item_const(&mut self, i: &'ast syn::ItemConst) {
        let k = if self.impl_self.is_empty() { if self.fns.is_empty() { "module" } else { "in_fn" } } else { "assoc" };
        self.hit(format!("item.const.{k}"), i.ident.span());
        self.hit(format!("const.type.{}", short_type(&i.ty)), i.ident.span());
        visit::visit_item_const(self, i);
    }

    fn visit_impl_item_const(&mut self, i: &'ast syn::ImplItemConst) {
        self.hit("item.const.assoc", i.ident.span());
        visit::visit_impl_item_const(self, i);
    }

    fn visit_item_static(&mut self, i: &'ast syn::ItemStatic) {
        let sp = i.ident.span();
        self.hit("item.static", sp);
        if matches!(i.mutability, syn::StaticMutability::Mut(_)) {
            self.hit("item.static.mut", sp);
        }
        self.hit(format!("static.type.{}", short_type(&i.ty)), sp);
        visit::visit_item_static(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        let test = is_cfg_test(&i.attrs);
        if test {
            self.inline_test(i.span());
            self.cats.push(Cat::Test);
        }
        self.hit(if i.content.is_some() { "item.mod.inline" } else { "item.mod.file" }, i.ident.span());
        if i.attrs.iter().any(|a| a.path().is_ident("path")) {
            self.hit("item.mod.path_attr", i.ident.span());
        }
        visit::visit_item_mod(self, i);
        if test {
            self.cats.pop();
        }
    }

    fn visit_item_use(&mut self, i: &'ast syn::ItemUse) {
        let sp = i.use_token.span;
        let mut out = Vec::new();
        flatten_use(&i.tree, String::new(), &mut out);
        self.hit("item.use", sp);
        if matches!(i.vis, syn::Visibility::Public(_) | syn::Visibility::Restricted(_)) {
            self.hit("use.reexport", sp);
        }
        fn has_group(t: &syn::UseTree) -> bool {
            match t {
                syn::UseTree::Path(p) => has_group(&p.tree),
                syn::UseTree::Group(_) => true,
                _ => false,
            }
        }
        if has_group(&i.tree) {
            self.hit("use.group", sp);
        }
        fn has_rename(t: &syn::UseTree) -> (bool, bool) {
            match t {
                syn::UseTree::Path(p) => has_rename(&p.tree),
                syn::UseTree::Rename(r) => (r.rename != "_", r.rename == "_"),
                syn::UseTree::Group(g) => g.items.iter().map(has_rename).fold((false, false), |a, b| (a.0 || b.0, a.1 || b.1)),
                _ => (false, false),
            }
        }
        let (ren, under) = has_rename(&i.tree);
        if ren {
            self.hit("use.rename", sp);
        }
        if under {
            self.hit("use.underscore", sp);
        }
        for (name, path) in out {
            let root = path.split("::").next().unwrap_or("");
            if name == "*" {
                let k = if path.ends_with("prelude") {
                    "prelude"
                } else if root == "super" || root == "self" {
                    "super_or_self"
                } else if STD_ROOTS.contains(&root) || self.g.ext_crates.contains(root) {
                    "external"
                } else {
                    "crate"
                };
                self.hit(format!("use.glob.{k}"), sp);
                if k == "external" {
                    self.hit(format!("use.glob_path.{path}"), sp);
                }
            } else {
                self.hit("use.name", sp);
                if STD_ROOTS.contains(&root) || self.g.ext_crates.contains(root) {
                    let norm = if STD_ROOTS.contains(&root) { format!("std{}", &path[root.len()..]) } else { path.clone() };
                    let kind = if STD_ROOTS.contains(&root) { "std" } else { "ext" };
                    self.hit(format!("{kind}.use.{norm}"), sp);
                }
            }
        }
    }

    fn visit_item_extern_crate(&mut self, i: &'ast syn::ItemExternCrate) {
        self.hit(format!("item.extern_crate.{}", i.ident), i.ident.span());
    }

    fn visit_item_foreign_mod(&mut self, i: &'ast syn::ItemForeignMod) {
        self.hit("item.foreign_mod", i.abi.extern_token.span);
    }

    fn visit_item_trait_alias(&mut self, i: &'ast syn::ItemTraitAlias) {
        self.hit("item.trait_alias", i.ident.span());
    }

    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
        if i.mac.path.is_ident("macro_rules") {
            self.hit("item.macro_rules", i.mac.path.span());
            return;
        }
        for a in &i.attrs {
            self.visit_attribute(a);
        }
        self.visit_macro(&i.mac);
    }

    fn visit_attribute(&mut self, a: &'ast syn::Attribute) {
        let p = a.path();
        let name = p.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
        if name == "doc" {
            return;
        }
        let sp = a.span();
        let inner = if matches!(a.style, syn::AttrStyle::Inner(_)) { "inner." } else { "" };
        self.hit(format!("attr.{inner}{name}"), sp);
        match name.as_str() {
            "cfg" | "cfg_attr" | "inline" | "repr" | "allow" | "expect" | "warn" | "deny" | "must_use" => {
                let args = match &a.meta {
                    syn::Meta::List(l) => l.tokens.to_string().replace(' ', ""),
                    syn::Meta::NameValue(_) => "value".into(),
                    syn::Meta::Path(_) => String::new(),
                };
                let args = if name == "cfg_attr" { args.split(',').next().unwrap_or("").to_string() } else { args };
                if name == "allow" || name == "expect" || name == "warn" || name == "deny" {
                    for l in args.split(',') {
                        self.hit(format!("{name}.{l}"), sp);
                    }
                } else {
                    self.hit(format!("{name}.{args}"), sp);
                }
            }
            _ => {}
        }
    }

    // ---------- types ----------
    fn visit_type(&mut self, t: &'ast syn::Type) {
        let sp = t.span();
        match t {
            syn::Type::Array(_) => self.hit("type.array", sp),
            syn::Type::Slice(_) => self.hit("type.slice", sp),
            syn::Type::Tuple(tt) => self.hit(if tt.elems.is_empty() { "type.unit" } else { "type.tuple" }, sp),
            syn::Type::Never(_) => self.hit("type.never", sp),
            syn::Type::Ptr(_) => self.hit("type.raw_ptr", sp),
            syn::Type::Infer(_) => self.hit("type.infer", sp),
            syn::Type::Macro(_) => self.hit("type.macro", sp),
            syn::Type::BareFn(f) => {
                self.hit("type.fn_ptr", sp);
                // Lifetimes in the signature (W-BORROW3). Lust reads a
                // `for<'a>` binder that means what elision means
                // (`lt.hrtb.fn_ptr`, else `lt.hrtb.noncanon`), keeps
                // `&'static`, and takes one lifetime of the enclosing item
                // in fn signatures and `let` types (`type.fn_ptr.lifetime`);
                // in fields and aliases it is `.field`, two lifetimes, a
                // nested one, or a `'static` type arg `.complex`.
                let binder = binder_names(f.lifetimes.as_ref());
                let ins: Vec<&syn::Type> = f.inputs.iter().map(|a| &a.ty).collect();
                let out = match &f.output {
                    syn::ReturnType::Type(_, t) => Some(&**t),
                    syn::ReturnType::Default => None,
                };
                if !binder.is_empty() {
                    self.hit(if binder_canonical(&binder, &ins, out) { "lt.hrtb.fn_ptr" } else { "lt.hrtb.noncanon" }, sp);
                }
                let mut sc = LtScan::default();
                for t in ins.iter().copied().chain(out) {
                    lt_scan(t, &binder, true, &mut sc);
                }
                if sc.complex || sc.free.len() > 1 {
                    self.hit("type.fn_ptr.lifetime.complex", sp);
                } else if !sc.free.is_empty() {
                    let sig_or_let = matches!(self.pos, Pos::Arg | Pos::Ret) || (!self.fns.is_empty() && matches!(self.pos, Pos::Other));
                    self.hit(if sig_or_let { "type.fn_ptr.lifetime" } else { "type.fn_ptr.lifetime.field" }, sp);
                }
            }
            syn::Type::ImplTrait(it) => {
                let pos = match self.pos {
                    Pos::Arg => "arg",
                    Pos::Ret => "ret",
                    _ => "other",
                };
                self.hit(format!("type.impl_trait.{pos}"), sp);
                for b in &it.bounds {
                    if let syn::TypeParamBound::Trait(tb) = b {
                        let k = self.path_key(&tb.path);
                        self.hit(format!("impl_trait.{pos}.{k}"), sp);
                    }
                }
            }
            syn::Type::TraitObject(o) => {
                self.hit("type.dyn", sp);
                for b in &o.bounds {
                    match b {
                        syn::TypeParamBound::Trait(tb) => {
                            let k = self.path_key(&tb.path);
                            self.hit(format!("dyn.{k}"), sp);
                            if tb.lifetimes.is_some() {
                                self.hit(if sugar_binder_canonical(tb) { "lt.hrtb.dyn" } else { "lt.hrtb.noncanon" }, sp);
                            }
                        }
                        syn::TypeParamBound::Lifetime(l) => {
                            self.hit(if l.ident == "static" { "dyn.lifetime_static" } else { "dyn.lifetime_named" }, sp);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        let old = self.pos;
        visit::visit_type(self, t);
        self.pos = old;
    }

    fn visit_type_reference(&mut self, r: &'ast syn::TypeReference) {
        let sp = r.and_token.span;
        let lk = match &r.lifetime {
            None => "elided",
            Some(l) if l.ident == "static" => "static",
            Some(l) if l.ident == "_" => "anon",
            Some(_) => "named",
        };
        let pos = match self.pos {
            Pos::Arg => "arg",
            Pos::Ret => "ret",
            Pos::Field => "field",
            Pos::Other => "other",
        };
        self.hit(format!("lt.ref.{lk}"), sp);
        self.hit(format!("lt.ref_pos.{pos}.{lk}"), sp);
        if lk == "static" || lk == "named" {
            self.hit(format!("lt.{lk}_ref_to.{}", short_type(&r.elem)), sp);
        }
        self.hit(if r.mutability.is_some() { "type.ref_mut" } else { "type.ref" }, sp);
        match &*r.elem {
            syn::Type::TraitObject(_) => self.hit(if r.mutability.is_some() { "dyn_in.&mut" } else { "dyn_in.&" }, sp),
            syn::Type::Path(p) if p.path.is_ident("str") => self.hit("type.ref_str", sp),
            syn::Type::Slice(_) => self.hit("type.ref_slice", sp),
            _ => {}
        }
        visit::visit_type_reference(self, r);
    }

    fn visit_type_path(&mut self, t: &'ast syn::TypePath) {
        let sp = t.span();
        if t.qself.is_some() {
            self.hit("type.qself_projection", sp);
        } else if t.path.segments.len() > 1 && self.in_tparams(&t.path.segments[0].ident.to_string()) {
            self.hit("type.assoc_projection", sp);
        }
        let name = last_ident(&t.path);
        match self.resolve(&t.path) {
            Some(r) => {
                let kind = if r.starts_with("std") { "std" } else { "ext" };
                self.hit(format!("{kind}.type.{r}"), sp);
                // HashMap<K, V, S> with an explicit hasher type arg.
                // The plain type key cannot tell this form apart, and
                // a custom-hasher map stays blocked after a plain
                // HashMap lands, so record the arg count here.
                if (name == "HashMap" || name == "HashSet") && r.contains("::collections::") {
                    if let Some(syn::PathArguments::AngleBracketed(a)) = t.path.segments.last().map(|s| &s.arguments) {
                        let types = a.args.iter().filter(|x| matches!(x, syn::GenericArgument::Type(_))).count();
                        if types > if name == "HashMap" { 2 } else { 1 } {
                            self.hit("map.hasher_arg", sp);
                        }
                    }
                }
                if WRAPPERS.contains(&name.as_str()) {
                    if let Some(syn::PathArguments::AngleBracketed(a)) = t.path.segments.last().map(|s| &s.arguments) {
                        for arg in &a.args {
                            if let syn::GenericArgument::Type(inner) = arg {
                                match inner {
                                    syn::Type::Path(ip) => {
                                        let iname = last_ident(&ip.path);
                                        if WRAPPERS.contains(&iname.as_str())
                                            || ["HashMap", "FxHashMap", "IndexMap", "VecDeque", "BTreeMap", "String"]
                                                .contains(&iname.as_str())
                                        {
                                            self.hit(format!("nest.{name}<{iname}>"), sp);
                                        }
                                    }
                                    syn::Type::TraitObject(_) => self.hit(format!("dyn_in.{name}"), sp),
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            None => {
                if t.path.segments.len() == 1 && PRIMS.contains(&name.as_str()) && t.qself.is_none() {
                    self.hit(format!("prim.{name}"), sp);
                } else if self.g.lt_types.contains_key(&name) {
                    self.hit(format!("lt.type_use.{name}"), sp);
                    self.hit("lt.type_use", sp);
                }
            }
        }
        for seg in &t.path.segments {
            match &seg.arguments {
                syn::PathArguments::AngleBracketed(a) => {
                    for arg in &a.args {
                        match arg {
                            syn::GenericArgument::Lifetime(l) => {
                                let lk = if l.ident == "static" {
                                    "static"
                                } else if l.ident == "_" {
                                    "anon"
                                } else {
                                    "named"
                                };
                                self.hit(format!("lt.path_arg.{lk}"), sp);
                                self.hit(format!("lt.carrier.{}", seg.ident), sp);
                            }
                            syn::GenericArgument::AssocType(_) => self.hit("type.assoc_type_binding", sp),
                            syn::GenericArgument::Const(_) => self.hit("type.const_arg", sp),
                            _ => {}
                        }
                    }
                }
                syn::PathArguments::Parenthesized(_) => {}
                syn::PathArguments::None => {}
            }
        }
        visit::visit_type_path(self, t);
    }

    /// `Fn(A) -> B` sugar, in types, bounds, `impl` and `dyn`. With a
    /// reference input it is higher-ranked without `for<'a>`.
    fn visit_path_segment(&mut self, seg: &'ast syn::PathSegment) {
        if let syn::PathArguments::Parenthesized(pa) = &seg.arguments {
            let sp = seg.ident.span();
            self.hit("type.fn_sugar", sp);
            let mut ts = TypeScan::default();
            for i in &pa.inputs {
                ts.visit_type(i);
            }
            if ts.refs > 0 || ts.anon > 0 {
                self.hit("lt.implicit_hrtb", sp);
                self.hit(format!("lt.implicit_hrtb.{}", seg.ident), sp);
            }
        }
        visit::visit_path_segment(self, seg);
    }

    fn visit_field(&mut self, f: &'ast syn::Field) {
        let old = self.pos;
        self.pos = Pos::Field;
        visit::visit_field(self, f);
        self.pos = old;
    }

    // ---------- expressions ----------
    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        let name = m.method.to_string();
        let sp = m.method.span();
        self.hit(format!("method.{name}"), sp);
        if let Some(tf) = &m.turbofish {
            self.hit(format!("method.turbofish.{name}"), sp);
            if name == "collect" {
                if let Some(syn::GenericArgument::Type(t)) = tf.args.first() {
                    self.hit(format!("collect.target.{}", short_type(t)), sp);
                }
            }
        }
        // Key by source position, not node address: macro arguments are
        // visited as owned temporaries, and a freed node's address can
        // be reused by the next temporary, which made counts wobble.
        if !self.receivers.contains(&sp.byte_range().start) {
            self.method_chain(m);
        }
        if let syn::Expr::MethodCall(r) = &*m.receiver {
            self.receivers.insert(r.method.span().byte_range().start);
        }
        if let syn::Expr::Try(t) = &*m.receiver {
            if let syn::Expr::MethodCall(r) = &*t.expr {
                self.receivers.insert(r.method.span().byte_range().start);
            }
        }
        for a in &m.args {
            match a {
                syn::Expr::Closure(_) => {
                    let ctx =
                        if ITER_ADAPTERS.contains(&name.as_str()) || ITER_CONSUMERS.contains(&name.as_str()) { "iter" } else { "other" };
                    self.hit(format!("closure.ctx.method.{ctx}"), sp);
                    self.hit(format!("closure.arg_of.{name}"), sp);
                }
                syn::Expr::Path(p)
                    if p.path.segments.len() >= 2
                        || p.path.segments[0].ident.to_string().chars().next().is_some_and(|c| c.is_lowercase()) =>
                {
                    if ITER_ADAPTERS.contains(&name.as_str())
                        || ITER_CONSUMERS.contains(&name.as_str())
                        || name.starts_with("map")
                        || name.starts_with("and_then")
                        || name.starts_with("unwrap_or_else")
                    {
                        self.hit("fn_item.as_closure_arg", sp);
                    }
                }
                _ => {}
            }
        }
        visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_call(&mut self, c: &'ast syn::ExprCall) {
        let sp = c.paren_token.span.join();
        let callee = match &*c.func {
            syn::Expr::Path(p) => {
                let segs: Vec<String> = p.path.segments.iter().map(|s| s.ident.to_string()).collect();
                let n = segs.len();
                segs[n.saturating_sub(2)..].join("::")
            }
            syn::Expr::Field(_) => {
                self.hit("expr.call.field_fn", sp);
                "field".into()
            }
            syn::Expr::Paren(p) if matches!(&*p.expr, syn::Expr::Field(_)) => {
                self.hit("expr.call.field_fn", sp);
                "field".into()
            }
            _ => {
                self.hit("expr.call.other_callee", sp);
                "other".into()
            }
        };
        for a in &c.args {
            if let syn::Expr::Closure(_) = a {
                self.hit("closure.ctx.call", sp);
                self.hit(format!("closure.arg_of_call.{callee}"), sp);
            }
        }
        visit::visit_expr_call(self, c);
    }

    fn visit_expr_path(&mut self, p: &'ast syn::ExprPath) {
        let sp = p.span();
        if p.qself.is_some() {
            self.hit("expr.qself_path", sp);
        }
        match self.resolve(&p.path) {
            Some(r) => {
                let kind = if r.starts_with("std") { "std" } else { "ext" };
                self.hit(format!("{kind}.expr.{r}"), sp);
            }
            None => {
                let first = p.path.segments[0].ident.to_string();
                if p.path.segments.len() > 1 && PRIMS.contains(&first.as_str()) {
                    let s: Vec<String> = p.path.segments.iter().map(|s| s.ident.to_string()).collect();
                    self.hit(format!("prim.path.{}", s.join("::")), sp);
                }
                if p.path.segments.len() > 1 && self.in_tparams(&first) {
                    self.hit("expr.generic_assoc_call", sp);
                }
            }
        }
        visit::visit_expr_path(self, p);
    }

    fn visit_expr_closure(&mut self, c: &'ast syn::ExprClosure) {
        let sp = c.or1_token.span;
        self.hit("closure", sp);
        if c.capture.is_some() {
            self.hit("closure.move", sp);
        }
        if c.asyncness.is_some() {
            self.hit("closure.async", sp);
        }
        if c.inputs.iter().any(|p| matches!(p, syn::Pat::Type(_))) {
            self.hit("closure.typed_params", sp);
        }
        if !matches!(c.output, syn::ReturnType::Default) {
            self.hit("closure.ret_type", sp);
        }
        self.hit(format!("closure.params.{}", c.inputs.len().min(4)), sp);
        let body = tok(&*c.body);
        if body.contains("self.") || body.contains("self)") || body.contains("(self") {
            self.hit("closure.uses_self", sp);
        }
        struct Assigns(bool);
        impl<'ast> Visit<'ast> for Assigns {
            fn visit_expr_assign(&mut self, _: &'ast syn::ExprAssign) {
                self.0 = true;
            }
            fn visit_expr_binary(&mut self, b: &'ast syn::ExprBinary) {
                if tok(&b.op).ends_with('=')
                    && !matches!(b.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_) | syn::BinOp::Le(_) | syn::BinOp::Ge(_))
                {
                    self.0 = true;
                }
                visit::visit_expr_binary(self, b);
            }
            fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}
        }
        let mut a = Assigns(false);
        a.visit_expr(&c.body);
        if a.0 {
            self.hit("closure.assigns", sp);
        }
        if c.inputs.iter().any(|p| !matches!(p, syn::Pat::Ident(_) | syn::Pat::Wild(_) | syn::Pat::Type(_))) {
            self.hit("closure.pattern_params", sp);
        }
        visit::visit_expr_closure(self, c);
    }

    fn visit_local(&mut self, l: &'ast syn::Local) {
        let sp = l.let_token.span;
        self.hit("stmt.let", sp);
        if let syn::Pat::Type(pt) = &l.pat {
            self.hit("stmt.let.typed", sp);
            if let Some(init) = &l.init {
                let mut e = &*init.expr;
                if let syn::Expr::Try(t) = e {
                    e = &t.expr;
                }
                if let syn::Expr::MethodCall(m) = e {
                    if m.method == "collect" && m.turbofish.is_none() {
                        self.hit(format!("collect.target.{}", short_type(&pt.ty)), sp);
                    }
                }
            }
        }
        if let Some(init) = &l.init {
            if init.diverge.is_some() {
                self.hit("expr.let_else", sp);
            }
            if matches!(&*init.expr, syn::Expr::Closure(_)) {
                self.hit("closure.ctx.let", sp);
            }
        } else {
            self.hit("stmt.let.uninit", sp);
        }
        visit::visit_local(self, l);
    }

    fn visit_expr_if(&mut self, i: &'ast syn::ExprIf) {
        let sp = i.if_token.span;
        self.hit("expr.if", sp);
        fn lets(e: &syn::Expr) -> usize {
            match e {
                syn::Expr::Let(_) => 1,
                syn::Expr::Binary(b) if matches!(b.op, syn::BinOp::And(_)) => lets(&b.left) + lets(&b.right),
                _ => 0,
            }
        }
        let n = lets(&i.cond);
        if matches!(&*i.cond, syn::Expr::Let(_)) {
            self.hit("expr.if_let", sp);
        } else if n > 0 {
            self.hit("expr.if_let_chain", sp);
        }
        visit::visit_expr_if(self, i);
    }

    fn visit_expr_while(&mut self, w: &'ast syn::ExprWhile) {
        let sp = w.while_token.span;
        self.hit(if matches!(&*w.cond, syn::Expr::Let(_)) { "expr.while_let" } else { "expr.while" }, sp);
        if w.label.is_some() {
            self.hit("expr.loop_label", sp);
        }
        visit::visit_expr_while(self, w);
    }

    fn visit_expr_match(&mut self, m: &'ast syn::ExprMatch) {
        let sp = m.match_token.span;
        self.hit("expr.match", sp);
        self.hit_n("match.arms", m.arms.len() as u64, sp);
        for a in &m.arms {
            if a.guard.is_some() {
                self.hit("match.guard", sp);
            }
        }
        visit::visit_expr_match(self, m);
    }

    fn visit_expr_for_loop(&mut self, f: &'ast syn::ExprForLoop) {
        let sp = f.for_token.span;
        self.hit("expr.for", sp);
        if f.label.is_some() {
            self.hit("expr.loop_label", sp);
        }
        let k = match &*f.expr {
            syn::Expr::Range(_) => "range".to_string(),
            syn::Expr::Reference(r) => {
                if r.mutability.is_some() {
                    "ref_mut".into()
                } else {
                    "ref".into()
                }
            }
            syn::Expr::MethodCall(m) => format!("method.{}", m.method),
            syn::Expr::Path(_) => "path".into(),
            syn::Expr::Field(_) => "field".into(),
            syn::Expr::Call(_) => "call".into(),
            _ => "other".into(),
        };
        self.hit(format!("for.over.{k}"), sp);
        visit::visit_expr_for_loop(self, f);
    }

    fn visit_expr_loop(&mut self, l: &'ast syn::ExprLoop) {
        self.hit("expr.loop", l.loop_token.span);
        if l.label.is_some() {
            self.hit("expr.loop_label", l.loop_token.span);
        }
        visit::visit_expr_loop(self, l);
    }

    fn visit_expr_break(&mut self, b: &'ast syn::ExprBreak) {
        let sp = b.break_token.span;
        if b.expr.is_some() {
            self.hit("expr.break_value", sp);
        }
        if b.label.is_some() {
            self.hit("expr.break_label", sp);
        }
        visit::visit_expr_break(self, b);
    }

    fn visit_expr_continue(&mut self, c: &'ast syn::ExprContinue) {
        if c.label.is_some() {
            self.hit("expr.continue_label", c.continue_token.span);
        }
    }

    fn visit_expr_try(&mut self, t: &'ast syn::ExprTry) {
        self.hit("expr.try_q", t.question_token.span);
        visit::visit_expr_try(self, t);
    }

    fn visit_expr_await(&mut self, a: &'ast syn::ExprAwait) {
        self.hit("expr.await", a.await_token.span);
        visit::visit_expr_await(self, a);
    }

    fn visit_expr_async(&mut self, a: &'ast syn::ExprAsync) {
        self.hit("expr.async", a.async_token.span);
        visit::visit_expr_async(self, a);
    }

    fn visit_expr_try_block(&mut self, t: &'ast syn::ExprTryBlock) {
        self.hit("expr.try_block", t.try_token.span);
        visit::visit_expr_try_block(self, t);
    }

    fn visit_expr_unsafe(&mut self, u: &'ast syn::ExprUnsafe) {
        self.hit("expr.unsafe", u.unsafe_token.span);
        visit::visit_expr_unsafe(self, u);
    }

    fn visit_expr_block(&mut self, b: &'ast syn::ExprBlock) {
        if b.label.is_some() {
            self.hit("expr.labeled_block", b.span());
        }
        visit::visit_expr_block(self, b);
    }

    fn visit_expr_const(&mut self, c: &'ast syn::ExprConst) {
        self.hit("expr.const_block", c.const_token.span);
        visit::visit_expr_const(self, c);
    }

    fn visit_expr_binary(&mut self, b: &'ast syn::ExprBinary) {
        self.hit(format!("op.{}", tok(&b.op)), b.op.span());
        visit::visit_expr_binary(self, b);
    }

    fn visit_expr_unary(&mut self, u: &'ast syn::ExprUnary) {
        let k = match u.op {
            syn::UnOp::Deref(_) => "deref",
            syn::UnOp::Not(_) => "not",
            syn::UnOp::Neg(_) => "neg",
            _ => "other",
        };
        self.hit(format!("op.unary.{k}"), u.op.span());
        visit::visit_expr_unary(self, u);
    }

    fn visit_expr_assign(&mut self, a: &'ast syn::ExprAssign) {
        self.hit("op.=", a.eq_token.span);
        visit::visit_expr_assign(self, a);
    }

    fn visit_expr_index(&mut self, i: &'ast syn::ExprIndex) {
        let sp = i.bracket_token.span.join();
        self.hit("expr.index", sp);
        if matches!(&*i.index, syn::Expr::Range(_)) {
            self.hit("expr.index.range", sp);
        }
        visit::visit_expr_index(self, i);
    }

    fn visit_expr_range(&mut self, r: &'ast syn::ExprRange) {
        let k = match (&r.start, &r.limits, &r.end) {
            (Some(_), syn::RangeLimits::HalfOpen(_), Some(_)) => "a..b",
            (Some(_), syn::RangeLimits::Closed(_), Some(_)) => "a..=b",
            (Some(_), _, None) => "a..",
            (None, syn::RangeLimits::HalfOpen(_), Some(_)) => "..b",
            (None, syn::RangeLimits::Closed(_), Some(_)) => "..=b",
            (None, _, None) => "..",
        };
        self.hit(format!("expr.range.{k}"), r.span());
        visit::visit_expr_range(self, r);
    }

    fn visit_expr_cast(&mut self, c: &'ast syn::ExprCast) {
        let sp = c.as_token.span;
        self.hit("expr.cast", sp);
        self.hit(format!("cast.to.{}", short_type(&c.ty)), sp);
        visit::visit_expr_cast(self, c);
    }

    fn visit_expr_reference(&mut self, r: &'ast syn::ExprReference) {
        self.hit(if r.mutability.is_some() { "expr.ref_mut" } else { "expr.ref" }, r.and_token.span);
        visit::visit_expr_reference(self, r);
    }

    fn visit_expr_struct(&mut self, s: &'ast syn::ExprStruct) {
        let sp = s.span();
        self.hit("expr.struct_lit", sp);
        if s.rest.is_some() {
            self.hit("expr.struct_lit.update_syntax", sp);
        }
        for f in &s.fields {
            if matches!(f.expr, syn::Expr::Closure(_)) {
                self.hit("closure.ctx.struct_field", sp);
            }
        }
        visit::visit_expr_struct(self, s);
    }

    fn visit_expr_tuple(&mut self, t: &'ast syn::ExprTuple) {
        if !t.elems.is_empty() {
            self.hit("expr.tuple", t.paren_token.span.join());
        }
        visit::visit_expr_tuple(self, t);
    }

    fn visit_expr_array(&mut self, a: &'ast syn::ExprArray) {
        self.hit("expr.array", a.bracket_token.span.join());
        visit::visit_expr_array(self, a);
    }

    fn visit_expr_repeat(&mut self, r: &'ast syn::ExprRepeat) {
        self.hit("expr.array_repeat", r.bracket_token.span.join());
        visit::visit_expr_repeat(self, r);
    }

    fn visit_expr_field(&mut self, f: &'ast syn::ExprField) {
        if matches!(f.member, syn::Member::Unnamed(_)) {
            self.hit("expr.tuple_field", f.dot_token.span);
        }
        visit::visit_expr_field(self, f);
    }

    fn visit_expr_return(&mut self, r: &'ast syn::ExprReturn) {
        self.hit("expr.return", r.return_token.span);
        visit::visit_expr_return(self, r);
    }

    fn visit_lit(&mut self, l: &'ast syn::Lit) {
        let sp = l.span();
        match l {
            syn::Lit::Float(f) => {
                self.hit("lit.float", sp);
                if !f.suffix().is_empty() {
                    self.hit(format!("lit.float.suffix.{}", f.suffix()), sp);
                }
            }
            syn::Lit::Int(i) => {
                if !i.suffix().is_empty() {
                    self.hit(format!("lit.int.suffix.{}", i.suffix()), sp);
                }
                let t = i.token().to_string();
                if t.starts_with("0x") || t.starts_with("0b") || t.starts_with("0o") {
                    self.hit("lit.int.radix", sp);
                }
                if i.base10_parse::<u64>().is_err() {
                    self.hit("lit.int.over_u64", sp);
                }
            }
            syn::Lit::Str(s) => {
                self.hit("lit.str", sp);
                if s.token().to_string().starts_with('r') {
                    self.hit("lit.str.raw", sp);
                }
            }
            syn::Lit::ByteStr(_) => self.hit("lit.byte_str", sp),
            syn::Lit::CStr(_) => self.hit("lit.c_str", sp),
            syn::Lit::Byte(_) => self.hit("lit.byte", sp),
            syn::Lit::Char(_) => self.hit("lit.char", sp),
            _ => {}
        }
    }

    // ---------- patterns ----------
    fn visit_pat(&mut self, p: &'ast syn::Pat) {
        let sp = p.span();
        match p {
            syn::Pat::Ident(i) => {
                if let Some(_) = &i.by_ref {
                    self.hit(if i.mutability.is_some() { "pat.ref_mut_binding" } else { "pat.ref_binding" }, sp);
                } else if i.mutability.is_some() {
                    self.hit("pat.mut_binding", sp);
                }
                if i.subpat.is_some() {
                    self.hit("pat.at_binding", sp);
                }
            }
            syn::Pat::Slice(s) => {
                self.hit("pat.slice", sp);
                if s.elems.iter().any(|e| {
                    matches!(e, syn::Pat::Rest(_))
                        || matches!(e, syn::Pat::Ident(i) if i.subpat.as_ref().is_some_and(|(_, sp)| matches!(**sp, syn::Pat::Rest(_))))
                }) {
                    self.hit("pat.slice.rest", sp);
                }
            }
            syn::Pat::Or(_) => self.hit("pat.or", sp),
            syn::Pat::Range(_) => {
                self.hit("pat.range", sp);
                return;
            }
            syn::Pat::Reference(_) => self.hit("pat.deref_ref", sp),
            syn::Pat::Tuple(_) => self.hit("pat.tuple", sp),
            syn::Pat::TupleStruct(t) => {
                self.hit("pat.tuple_struct", sp);
                if t.elems.iter().any(nested_enum_pat) {
                    self.hit("pat.nested_enum", sp);
                }
            }
            syn::Pat::Struct(s) => {
                self.hit("pat.struct", sp);
                if s.rest.is_some() {
                    self.hit("pat.struct.rest", sp);
                }
                if s.fields.iter().any(|f| nested_enum_pat(&f.pat)) {
                    self.hit("pat.nested_enum", sp);
                }
            }
            syn::Pat::Wild(_) => self.hit("pat.wild", sp),
            syn::Pat::Lit(_) => self.hit("pat.lit", sp),
            syn::Pat::Path(_) => self.hit("pat.path", sp),
            syn::Pat::Rest(_) => self.hit("pat.rest", sp),
            syn::Pat::Const(_) => self.hit("pat.const_block", sp),
            syn::Pat::Verbatim(v) => {
                let s = v.to_string();
                self.hit(if s.starts_with("box") { "pat.box" } else { "pat.verbatim" }, sp);
            }
            _ => {}
        }
        visit::visit_pat(self, p);
    }

    // ---------- macros ----------
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        let name = m.path.segments.iter().map(|s| s.ident.to_string()).collect::<Vec<_>>().join("::");
        let sp = m.path.span();
        self.hit(format!("macro.{name}"), sp);
        let last = last_ident(&m.path);
        let user = self.g.macro_names.contains(&last)
            && !["vec", "format", "write", "println", "matches", "assert", "panic"].contains(&last.as_str());
        if user {
            self.hit("macro.user_call", sp);
        }
        let tokens = m.tokens.clone();
        match last.as_str() {
            "matches" | "assert_matches" | "debug_assert_matches" => {
                let parser = |input: syn::parse::ParseStream| -> syn::Result<(syn::Expr, syn::Pat, Option<syn::Expr>)> {
                    let e: syn::Expr = input.parse()?;
                    input.parse::<syn::Token![,]>()?;
                    let p = syn::Pat::parse_multi_with_leading_vert(input)?;
                    let guard = if input.peek(syn::Token![if]) {
                        input.parse::<syn::Token![if]>()?;
                        Some(input.parse::<syn::Expr>()?)
                    } else {
                        None
                    };
                    let _ = input.parse::<Option<syn::Token![,]>>();
                    let _: proc_macro2::TokenStream = input.parse()?;
                    Ok((e, p, guard))
                };
                if let Ok((e, p, g)) = parser.parse2(tokens) {
                    self.visit_expr(&e);
                    self.visit_pat(&p);
                    if let Some(g) = g {
                        self.visit_expr(&g);
                    }
                }
            }
            "vec" => {
                let rep = |input: syn::parse::ParseStream| -> syn::Result<(syn::Expr, syn::Expr)> {
                    let a: syn::Expr = input.parse()?;
                    input.parse::<syn::Token![;]>()?;
                    let b: syn::Expr = input.parse()?;
                    Ok((a, b))
                };
                if let Ok((a, b)) = rep.parse2(tokens.clone()) {
                    self.hit("macro.vec.repeat", sp);
                    self.visit_expr(&a);
                    self.visit_expr(&b);
                } else if let Ok(list) = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated.parse2(tokens) {
                    self.visit_exprs(&list);
                }
            }
            "thread_local" => {
                if let Ok(f) = syn::parse2::<syn::File>(tokens) {
                    for it in &f.items {
                        self.visit_item(it);
                    }
                }
            }
            _ => {
                if let Ok(list) = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated.parse2(tokens) {
                    self.visit_exprs(&list);
                } else if user {
                    self.hit("macro.user_call.unparsed_args", sp);
                }
            }
        }
    }
}

fn nested_enum_pat(p: &syn::Pat) -> bool {
    match p {
        syn::Pat::TupleStruct(_) | syn::Pat::Struct(_) => true,
        syn::Pat::Path(pp) => pp.path.segments.len() > 1,
        syn::Pat::Reference(r) => nested_enum_pat(&r.pat),
        syn::Pat::Or(o) => o.cases.iter().any(nested_enum_pat),
        syn::Pat::Ident(i) => i.subpat.as_ref().is_some_and(|(_, s)| nested_enum_pat(s)),
        _ => false,
    }
}
