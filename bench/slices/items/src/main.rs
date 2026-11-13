//! slice-items <file.rs>: one line per removable item, `start end depth kind name`.
//! slice-items --tree <root.rs>: every file in the crate's module tree, one per line
//! (cfg is not evaluated: a file that exists is listed).
//! slice-items --bodies <file.rs>: every fn body (free fns, impl and trait methods),
//! `open_line open_col close_line close_col start_line` (1-based columns of the braces).
//! slice-items --tests <root.rs>: every `#[test]` fn in the module tree (scripts/tsrust_build.py):
//! `test <file> <fn line> <module path>::<name> <ignore 0|1> <returns a value 0|1>`, after it
//! one `mod <file> <line> <private 0|1> <name>` per module on its path (the line of `mod`).
//! slice-items --sites <file.rs>: edit sites for scripts/tsedit.py, one per line:
//!   `fn ol oc cl cc sig_line paren_l paren_c ret_l ret_c where_l where_c n_inputs flags name tparams depth`
//!   (body braces, the `)` that closes the inputs, the end of the return type, the `where`
//!   keyword; 0 0 when absent; flags: g generic type params, m inherent impl method, t trait
//!   impl method, d trait default body, s self receiver, r returns `impl Trait`, c const,
//!   a async, u unsafe, n nested in a fn body, x under a cfg or test attribute, b the body
//!   holds a type item (struct, enum, union, trait, type alias, impl), `-` for none;
//!   tparams: the fn's own type params, comma separated, `-` for none; depth: inline mod depth)
//!   `derive line depth kind name generic traits` (a one-line `#[derive(..)]` on a struct
//!   or enum; depth = inline mod depth, generic 0/1, traits comma separated)
//!
//! bench/slices/prune.py removes the innermost item that holds a lust or
//! rustc error. Removable units: items at file level and inside inline
//! `mod` blocks, and the items of inherent impls. A trait impl, a trait,
//! and a fn are removed whole. `start` includes the item's attributes and
//! doc comments. Lines are 1-based and inclusive.

use proc_macro2::Span;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::{ImplItem, Item};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 3 && args[1] == "--tree" {
        let root = Path::new(&args[2]).to_path_buf();
        let mut out = Vec::new();
        tree(&root, root.parent().unwrap().to_path_buf(), &[], &mut out);
        for (p, _) in out {
            println!("{}", p.display());
        }
        return;
    }
    if args.len() == 3 && args[1] == "--tests" {
        let root = Path::new(&args[2]).to_path_buf();
        let mut files = Vec::new();
        tree(&root, root.parent().unwrap().to_path_buf(), &[], &mut files);
        let mut out = String::new();
        for (p, steps) in &files {
            tests(&parse(p).items, p, steps, &mut out);
        }
        print!("{out}");
        return;
    }
    if args.len() == 3 && args[1] == "--sites" {
        let file = parse(Path::new(&args[2]));
        let mut v = Sites::default();
        syn::visit::visit_file(&mut v, &file);
        print!("{}", v.out);
        return;
    }
    if args.len() == 3 && args[1] == "--bodies" {
        let file = parse(Path::new(&args[2]));
        let mut v = Bodies(String::new());
        syn::visit::visit_file(&mut v, &file);
        print!("{}", v.0);
        return;
    }
    let path = args.get(1).expect("usage: slice-items <file.rs> | --tree <root.rs> | --bodies <file.rs>");
    let file = parse(Path::new(path));
    let mut out = String::new();
    items(&file.items, 0, &mut out);
    print!("{out}");
}

fn parse(path: &Path) -> syn::File {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    match syn::parse_file(&text) {
        Ok(f) => f,
        Err(e) => {
            let at = e.span().start();
            eprintln!("parse error at {}:{}:{}: {e}", path.display(), at.line, at.column + 1);
            std::process::exit(2);
        }
    }
}

/// The `#[path = "..."]` value of a module, if any.
fn path_attr(attrs: &[syn::Attribute]) -> Option<String> {
    attrs.iter().find(|a| a.path().is_ident("path")).and_then(|a| match &a.meta {
        syn::Meta::NameValue(nv) => match &nv.value {
            syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => Some(s.value()),
            _ => None,
        },
        _ => None,
    })
}

/// One module on the path from the crate root: its name and where its `mod` keyword is.
#[derive(Clone)]
struct Step {
    name: String,
    file: PathBuf,
    line: usize,
    private: bool,
}

fn step(m: &syn::ItemMod, file: &Path) -> Step {
    let private = matches!(m.vis, syn::Visibility::Inherited);
    Step { name: m.ident.to_string(), file: file.to_path_buf(), line: m.mod_token.span.start().line, private }
}

/// Adds `file` and every module file below it, each with its module path from the root (the
/// first one when a file is mounted twice). `dir` is where its child `mod x;` files live.
fn tree(file: &Path, dir: PathBuf, steps: &[Step], out: &mut Vec<(PathBuf, Vec<Step>)>) {
    if out.iter().any(|(p, _)| p == file) {
        return;
    }
    out.push((file.to_path_buf(), steps.to_vec()));
    let parsed = parse(file);
    mods(&parsed.items, file, &dir, false, steps, out);
}

/// `--tests`: the `#[test]` fns of one file's items, walking inline `mod` blocks.
fn tests(list: &[Item], file: &Path, steps: &[Step], out: &mut String) {
    for it in list {
        match it {
            Item::Fn(f) if f.attrs.iter().any(|a| a.path().is_ident("test")) => {
                let path: Vec<&str> = steps.iter().map(|s| s.name.as_str()).collect();
                let ignore = f.attrs.iter().any(|a| a.path().is_ident("ignore"));
                let value = !matches!(f.sig.output, syn::ReturnType::Default);
                let full = if path.is_empty() { f.sig.ident.to_string() } else { format!("{}::{}", path.join("::"), f.sig.ident) };
                out.push_str(&format!("test {} {} {full} {} {}\n", file.display(), f.sig.fn_token.span.start().line, ignore as u8, value as u8));
                for s in steps {
                    out.push_str(&format!("mod {} {} {} {}\n", s.file.display(), s.line, s.private as u8, s.name));
                }
            }
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    let mut sub = steps.to_vec();
                    sub.push(step(m, file));
                    tests(inner, file, &sub, out);
                }
            }
            _ => {}
        }
    }
}

/// `inline`: `list` is the body of an inline `mod m { .. }`. A `#[path]` there is relative to
/// `dir`; outside inline blocks it is relative to the directory of `file` (the Reference rule).
fn mods(list: &[Item], file: &Path, dir: &Path, inline: bool, steps: &[Step], out: &mut Vec<(PathBuf, Vec<Step>)>) {
    for it in list {
        let Item::Mod(m) = it else { continue };
        let name = m.ident.to_string();
        let mut steps = steps.to_vec();
        steps.push(step(m, file));
        if let Some((_, inner)) = &m.content {
            let sub = match path_attr(&m.attrs) {
                Some(p) => dir.join(p),
                None => dir.join(&name),
            };
            mods(inner, file, &sub, true, &steps, out);
            continue;
        }
        if let Some(p) = path_attr(&m.attrs) {
            // A #[path] file owns its directory: its children sit next to it.
            let f = if inline { dir.join(p) } else { file.parent().unwrap().join(p) };
            if f.exists() {
                let d = f.parent().unwrap().to_path_buf();
                tree(&f, d, &steps, out);
            }
            continue;
        }
        let flat = dir.join(format!("{name}.rs"));
        let modrs = dir.join(&name).join("mod.rs");
        if flat.exists() {
            tree(&flat, dir.join(&name), &steps, out);
        } else if modrs.exists() {
            tree(&modrs, dir.join(&name), &steps, out);
        }
    }
}

/// Name of an item-position macro call: the first identifier of its input after any
/// attributes and item keywords (`handle!(/// doc\n TypeId)` gives `TypeId`), so prune logs name what the call
/// defines. `-` when there is none.
fn call_name(mac: &syn::Macro) -> String {
    const KEYWORDS: &[&str] = &["pub", "static", "const", "enum", "struct", "fn", "type", "mut", "unsafe", "impl", "trait"];
    let mut toks = mac.tokens.clone().into_iter().peekable();
    while let Some(t) = toks.next() {
        match t {
            proc_macro2::TokenTree::Punct(p) if p.as_char() == '#' => {
                toks.next();
            }
            proc_macro2::TokenTree::Ident(i) if !KEYWORDS.contains(&i.to_string().as_str()) => return i.to_string(),
            proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Group(_) => {}
            _ => return "-".into(),
        }
    }
    "-".into()
}

/// First line of an item: its first attribute (doc comments are attributes) or the item.
fn start(attrs: &[syn::Attribute], item: Span) -> usize {
    attrs.iter().map(|a| a.span().start().line).chain([item.start().line]).min().unwrap()
}

fn emit(out: &mut String, start: usize, end: usize, depth: usize, kind: &str, name: &str) {
    out.push_str(&format!("{start} {end} {depth} {kind} {name}\n"));
}

fn items(list: &[Item], depth: usize, out: &mut String) {
    for it in list {
        let (attrs, kind, name): (&[syn::Attribute], &str, String) = match it {
            Item::Fn(f) => (&f.attrs, "fn", f.sig.ident.to_string()),
            Item::Struct(s) => (&s.attrs, "struct", s.ident.to_string()),
            Item::Enum(e) => (&e.attrs, "enum", e.ident.to_string()),
            Item::Union(u) => (&u.attrs, "union", u.ident.to_string()),
            Item::Trait(t) => (&t.attrs, "trait", t.ident.to_string()),
            Item::TraitAlias(t) => (&t.attrs, "trait", t.ident.to_string()),
            Item::Type(t) => (&t.attrs, "type", t.ident.to_string()),
            Item::Const(c) => (&c.attrs, "const", c.ident.to_string()),
            Item::Static(s) => (&s.attrs, "static", s.ident.to_string()),
            Item::Use(u) => (&u.attrs, "use", "-".into()),
            Item::Mod(m) => (&m.attrs, if m.content.is_some() { "mod" } else { "moddecl" }, m.ident.to_string()),
            Item::Impl(i) => (&i.attrs, if i.trait_.is_some() { "traitimpl" } else { "impl" }, "-".into()),
            Item::Macro(m) => (&m.attrs, "macro", m.ident.as_ref().map(|i| i.to_string()).unwrap_or_else(|| call_name(&m.mac))),
            Item::ExternCrate(e) => (&e.attrs, "extern", e.ident.to_string()),
            Item::ForeignMod(f) => (&f.attrs, "foreign", "-".into()),
            _ => (&[], "other", "-".into()),
        };
        let s = start(attrs, it.span());
        emit(out, s, it.span().end().line, depth, kind, &name);
        match it {
            Item::Mod(m) => {
                if let Some((_, inner)) = &m.content {
                    items(inner, depth + 1, out);
                }
            }
            Item::Impl(i) if i.trait_.is_none() => {
                for ii in &i.items {
                    let (attrs, kind, name): (&[syn::Attribute], &str, String) = match ii {
                        ImplItem::Fn(f) => (&f.attrs, "fn", f.sig.ident.to_string()),
                        ImplItem::Const(c) => (&c.attrs, "const", c.ident.to_string()),
                        ImplItem::Type(t) => (&t.attrs, "type", t.ident.to_string()),
                        ImplItem::Macro(m) => (&m.attrs, "macro", "-".into()),
                        _ => (&[], "other", "-".into()),
                    };
                    emit(out, start(attrs, ii.span()), ii.span().end().line, depth + 1, kind, &name);
                }
            }
            _ => {}
        }
    }
}

/// Collects fn bodies for `--bodies`: brace positions, and the fn's first line.
struct Bodies(String);

impl Bodies {
    fn body(&mut self, block: &syn::Block, start: usize) {
        let (o, c) = (block.brace_token.span.open().start(), block.brace_token.span.close().start());
        self.0.push_str(&format!("{} {} {} {} {start}\n", o.line, o.column + 1, c.line, c.column + 1));
    }
}

impl<'a> syn::visit::Visit<'a> for Bodies {
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        self.body(&f.block, f.sig.span().start().line);
        syn::visit::visit_item_fn(self, f);
    }
    fn visit_impl_item_fn(&mut self, f: &'a syn::ImplItemFn) {
        self.body(&f.block, f.sig.span().start().line);
        syn::visit::visit_impl_item_fn(self, f);
    }
    fn visit_trait_item_fn(&mut self, f: &'a syn::TraitItemFn) {
        if let Some(b) = &f.default {
            self.body(b, f.sig.span().start().line);
        }
        syn::visit::visit_trait_item_fn(self, f);
    }
}

/// True when a fn body holds a type item (struct, enum, union, trait, type alias, impl) at any
/// depth: the `b` site flag (tsedit's `blockbody` kind).
fn holds_type_item(block: &syn::Block) -> bool {
    struct Find(bool);
    impl<'a> syn::visit::Visit<'a> for Find {
        fn visit_item(&mut self, i: &'a Item) {
            self.0 |= matches!(i, Item::Struct(_) | Item::Enum(_) | Item::Union(_) | Item::Trait(_) | Item::Type(_) | Item::Impl(_));
            syn::visit::visit_item(self, i);
        }
    }
    let mut f = Find(false);
    syn::visit::visit_block(&mut f, block);
    f.0
}

/// Collects edit sites for `--sites` (scripts/tsedit.py picks its one-fn edits from them).
#[derive(Default)]
struct Sites {
    out: String,
    /// Inline `mod` depth, fn-body depth, and cfg/test attribute depth of the walk.
    mods: usize,
    fns: usize,
    cfg: usize,
    /// The enclosing impl: 'm' inherent, 't' trait impl; the enclosing trait: 'd'.
    owner: Option<char>,
}

/// True for `#[cfg(..)]`, `#[cfg_attr(..)]`, and `#[test]`.
fn cfg_like(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| a.path().is_ident("cfg") || a.path().is_ident("cfg_attr") || a.path().is_ident("test"))
}

fn lc(s: proc_macro2::LineColumn) -> String {
    format!("{} {}", s.line, s.column + 1)
}

impl Sites {
    fn site(&mut self, attrs: &[syn::Attribute], sig: &syn::Signature, block: &syn::Block) {
        let (o, c) = (block.brace_token.span.open().start(), block.brace_token.span.close().start());
        let paren = sig.paren_token.span.close().start();
        let ret = match &sig.output {
            syn::ReturnType::Type(_, ty) => lc(ty.span().end()),
            syn::ReturnType::Default => "0 0".into(),
        };
        let wh = sig.generics.where_clause.as_ref().map_or("0 0".into(), |w| lc(w.where_token.span.start()));
        let tparams: Vec<String> = sig.generics.type_params().map(|t| t.ident.to_string()).collect();
        let mut flags = String::new();
        let mut flag = |on: bool, f: char| {
            if on {
                flags.push(f)
            }
        };
        flag(!tparams.is_empty(), 'g');
        if let Some(k) = self.owner {
            flag(true, k);
        }
        flag(sig.receiver().is_some(), 's');
        flag(matches!(&sig.output, syn::ReturnType::Type(_, ty) if matches!(**ty, syn::Type::ImplTrait(_))), 'r');
        flag(sig.constness.is_some(), 'c');
        flag(sig.asyncness.is_some(), 'a');
        flag(sig.unsafety.is_some(), 'u');
        flag(self.fns > 0, 'n');
        flag(self.cfg > 0 || cfg_like(attrs), 'x');
        flag(holds_type_item(block), 'b');
        let flags = if flags.is_empty() { "-".into() } else { flags };
        let tp = if tparams.is_empty() { "-".into() } else { tparams.join(",") };
        self.out.push_str(&format!(
            "fn {} {} {} {} {} {} {} {flags} {} {tp} {}\n",
            lc(o),
            lc(c),
            sig.span().start().line,
            lc(paren),
            ret,
            wh,
            sig.inputs.len(),
            sig.ident,
            self.mods
        ));
    }

    fn derive(&mut self, attrs: &[syn::Attribute], kind: &str, name: &syn::Ident, generics: &syn::Generics) {
        for a in attrs.iter().filter(|a| a.path().is_ident("derive")) {
            let (s, e) = (a.span().start(), a.span().end());
            let Ok(paths) = a.parse_args_with(syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated) else {
                continue;
            };
            if s.line != e.line || paths.is_empty() {
                continue;
            }
            let traits: Vec<String> =
                paths.iter().map(|p| p.segments.iter().map(|g| g.ident.to_string()).collect::<Vec<_>>().join("::")).collect();
            let generic = generics.params.iter().any(|p| !matches!(p, syn::GenericParam::Lifetime(_))) as u8;
            let x = if self.cfg > 0 || self.fns > 0 || cfg_like(attrs) { "x" } else { "" };
            self.out.push_str(&format!("derive {} {} {kind}{x} {name} {generic} {}\n", s.line, self.mods, traits.join(",")));
        }
    }

    /// Runs `f` one level deeper in the cfg count when `attrs` has a cfg-like attribute.
    fn under<F: FnOnce(&mut Self)>(&mut self, attrs: &[syn::Attribute], f: F) {
        let c = cfg_like(attrs) as usize;
        self.cfg += c;
        f(self);
        self.cfg -= c;
    }
}

impl<'a> syn::visit::Visit<'a> for Sites {
    fn visit_item_mod(&mut self, m: &'a syn::ItemMod) {
        self.mods += 1;
        self.under(&m.attrs, |v| syn::visit::visit_item_mod(v, m));
        self.mods -= 1;
    }
    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        let prev = self.owner.replace(if i.trait_.is_some() { 't' } else { 'm' });
        self.under(&i.attrs, |v| syn::visit::visit_item_impl(v, i));
        self.owner = prev;
    }
    fn visit_item_trait(&mut self, t: &'a syn::ItemTrait) {
        let prev = self.owner.replace('d');
        self.under(&t.attrs, |v| syn::visit::visit_item_trait(v, t));
        self.owner = prev;
    }
    fn visit_item_struct(&mut self, s: &'a syn::ItemStruct) {
        self.derive(&s.attrs, "struct", &s.ident, &s.generics);
        self.under(&s.attrs, |v| syn::visit::visit_item_struct(v, s));
    }
    fn visit_item_enum(&mut self, e: &'a syn::ItemEnum) {
        self.derive(&e.attrs, "enum", &e.ident, &e.generics);
        self.under(&e.attrs, |v| syn::visit::visit_item_enum(v, e));
    }
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        let prev = self.owner.take();
        self.site(&f.attrs, &f.sig, &f.block);
        self.fns += 1;
        self.under(&f.attrs, |v| syn::visit::visit_item_fn(v, f));
        self.fns -= 1;
        self.owner = prev;
    }
    fn visit_impl_item_fn(&mut self, f: &'a syn::ImplItemFn) {
        self.site(&f.attrs, &f.sig, &f.block);
        let prev = self.owner.take();
        self.fns += 1;
        self.under(&f.attrs, |v| syn::visit::visit_impl_item_fn(v, f));
        self.fns -= 1;
        self.owner = prev;
    }
    fn visit_trait_item_fn(&mut self, f: &'a syn::TraitItemFn) {
        if let Some(b) = &f.default {
            self.site(&f.attrs, &f.sig, b);
        }
        let prev = self.owner.take();
        self.fns += 1;
        self.under(&f.attrs, |v| syn::visit::visit_trait_item_fn(v, f));
        self.fns -= 1;
        self.owner = prev;
    }
}
