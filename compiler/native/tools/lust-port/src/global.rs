//! Pass 1: facts about the whole tree that pass 2 needs (types with
//! lifetime parameters, macro_rules definitions, prelude re-exports, user
//! trait and type names).

use crate::walk::{Cat, Files};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use syn::spanned::Spanned;

#[allow(dead_code)]
pub struct LtType {
    pub file: u32,
    pub cat: Cat,
    pub kind: &'static str,
    pub line: usize,
    pub lt_params: usize,
    pub type_params: usize,
    /// Fields whose type is a reference (`&T`), not counting `&'static`.
    pub ref_fields: usize,
    pub static_ref_fields: usize,
    pub mut_ref_fields: usize,
}

#[allow(dead_code)] // fields beyond the counts stay for debugging and analyzer parity
pub struct MacroDef {
    pub name: String,
    pub file: u32,
    pub cat: Cat,
    pub line: usize,
    pub lines: usize,
    pub rules: usize,
    pub exported: bool,
    /// What the expansion makes: item keywords found in the bodies.
    pub makes: BTreeSet<String>,
    pub fragments: BTreeSet<String>,
    pub repetition: bool,
    pub recursive: bool,
}

pub struct Global {
    /// External crate root names (seed list plus the scanned packages'
    /// dependency names, with `_` spelling).
    pub ext_crates: HashSet<String>,
    pub lt_types: BTreeMap<String, LtType>,
    pub macro_defs: Vec<MacroDef>,
    pub macro_names: HashSet<String>,
    /// name -> full external path, from the crate preludes.
    pub prelude: HashMap<String, String>,
    pub user_traits: HashSet<String>,
    pub user_types: HashSet<String>,
}

pub const EXT_CRATES: &[&str] = &[
    "rustc_hash",
    "indexmap",
    "memchr",
    "smallvec",
    "bumpalo",
    "xxhash_rust",
    "flate2",
    "jiff",
    "rustix",
    "signal_hook",
    "tikv_jemallocator",
    "tikv_jemalloc_ctl",
    "nix",
    "num_bigint",
    "num_traits",
    "ryu_js",
    "lzma_rust2",
    "unicode_normalization",
    "name_to_handle_at",
    "notify",
    "miow",
    "same_file",
    "libc",
    "serde",
    "serde_json",
];
pub const STD_ROOTS: &[&str] = &["std", "core", "alloc"];

pub fn collect(files: &Files, ext_crates: &HashSet<String>) -> Global {
    let mut g = Global {
        ext_crates: ext_crates.clone(),
        lt_types: BTreeMap::new(),
        macro_defs: Vec::new(),
        macro_names: HashSet::new(),
        prelude: HashMap::new(),
        user_traits: HashSet::new(),
        user_types: HashSet::new(),
    };
    for (i, f) in files.files.iter().enumerate() {
        let Some(ast) = &f.ast else { continue };
        let mut v = Pass1 { g: &mut g, file: i as u32, cat: f.cat };
        syn::visit::Visit::visit_file(&mut v, ast);
        if f.rel.ends_with("prelude.rs") {
            for item in &ast.items {
                if let syn::Item::Use(u) = item {
                    let mut out = Vec::new();
                    crate::analyze::flatten_use(&u.tree, String::new(), &mut out);
                    for (name, path) in out {
                        let root = path.split("::").next().unwrap_or("");
                        if name != "*" && name != "_" && (STD_ROOTS.contains(&root) || g.ext_crates.contains(root)) {
                            g.prelude.insert(name, path);
                        }
                    }
                }
            }
        }
    }
    g
}

struct Pass1<'g> {
    g: &'g mut Global,
    file: u32,
    cat: Cat,
}

/// (non-static refs, `&'static` refs, `&mut` refs) among the fields.
fn count_ref_fields(fields: &syn::Fields) -> (usize, usize, usize) {
    let (mut r, mut s, mut m) = (0, 0, 0);
    for f in fields {
        if let syn::Type::Reference(t) = &f.ty {
            if t.lifetime.as_ref().is_some_and(|l| l.ident == "static") {
                s += 1;
            } else {
                r += 1;
            }
            if t.mutability.is_some() {
                m += 1;
            }
        }
    }
    (r, s, m)
}

impl Pass1<'_> {
    fn lt_type(&mut self, name: String, kind: &'static str, gens: &syn::Generics, line: usize, refs: (usize, usize, usize)) {
        self.g.user_types.insert(name.clone());
        let lts = gens.lifetimes().count();
        if lts == 0 && refs.0 == 0 {
            return;
        }
        self.g.lt_types.insert(
            name,
            LtType {
                file: self.file,
                cat: self.cat,
                kind,
                line,
                lt_params: lts,
                type_params: gens.type_params().count(),
                ref_fields: refs.0,
                static_ref_fields: refs.1,
                mut_ref_fields: refs.2,
            },
        );
    }
}

impl<'ast> syn::visit::Visit<'ast> for Pass1<'_> {
    fn visit_item_struct(&mut self, i: &'ast syn::ItemStruct) {
        let refs = count_ref_fields(&i.fields);
        self.lt_type(i.ident.to_string(), "struct", &i.generics, i.ident.span().start().line, refs);
        syn::visit::visit_item_struct(self, i);
    }
    fn visit_item_enum(&mut self, i: &'ast syn::ItemEnum) {
        let mut refs = (0, 0, 0);
        for v in &i.variants {
            let r = count_ref_fields(&v.fields);
            refs.0 += r.0;
            refs.1 += r.1;
            refs.2 += r.2;
        }
        self.lt_type(i.ident.to_string(), "enum", &i.generics, i.ident.span().start().line, refs);
        syn::visit::visit_item_enum(self, i);
    }
    fn visit_item_union(&mut self, i: &'ast syn::ItemUnion) {
        self.lt_type(i.ident.to_string(), "union", &i.generics, i.ident.span().start().line, (0, 0, 0));
    }
    fn visit_item_type(&mut self, i: &'ast syn::ItemType) {
        let mut refs = (0, 0, 0);
        if let syn::Type::Reference(t) = &*i.ty {
            if t.lifetime.as_ref().is_some_and(|l| l.ident == "static") {
                refs.1 = 1;
            }
        }
        self.lt_type(i.ident.to_string(), "type", &i.generics, i.ident.span().start().line, (0, refs.1, 0));
    }
    fn visit_item_trait(&mut self, i: &'ast syn::ItemTrait) {
        self.g.user_traits.insert(i.ident.to_string());
        if i.generics.lifetimes().count() > 0 {
            self.lt_type(i.ident.to_string(), "trait", &i.generics, i.ident.span().start().line, (0, 0, 0));
        }
        syn::visit::visit_item_trait(self, i);
    }
    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
        if let Some(ident) = &i.ident {
            if i.mac.path.is_ident("macro_rules") {
                let name = ident.to_string();
                let toks: Vec<proc_macro2::TokenTree> = i.mac.tokens.clone().into_iter().collect();
                let rules = toks
                    .windows(2)
                    .filter(|w| {
                        matches!(&w[0], proc_macro2::TokenTree::Punct(p) if p.as_char() == '=')
                            && matches!(&w[1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '>')
                    })
                    .count();
                let mut makes = BTreeSet::new();
                let mut fragments = BTreeSet::new();
                let mut repetition = false;
                let mut recursive = false;
                scan_tokens(i.mac.tokens.clone(), &name, &mut makes, &mut fragments, &mut repetition, &mut recursive);
                let span = i.span();
                self.g.macro_defs.push(MacroDef {
                    name: name.clone(),
                    file: self.file,
                    cat: self.cat,
                    line: span.start().line,
                    lines: span.end().line - span.start().line + 1,
                    rules,
                    exported: i.attrs.iter().any(|a| a.path().is_ident("macro_export")),
                    makes,
                    fragments,
                    repetition,
                    recursive,
                });
                self.g.macro_names.insert(name);
            }
        }
    }
}

fn scan_tokens(
    ts: proc_macro2::TokenStream,
    self_name: &str,
    makes: &mut BTreeSet<String>,
    fragments: &mut BTreeSet<String>,
    repetition: &mut bool,
    recursive: &mut bool,
) {
    let toks: Vec<proc_macro2::TokenTree> = ts.into_iter().collect();
    for (k, t) in toks.iter().enumerate() {
        match t {
            proc_macro2::TokenTree::Ident(id) => {
                let s = id.to_string();
                if ["impl", "fn", "struct", "enum", "const", "static", "trait", "mod", "type", "macro_rules"].contains(&s.as_str()) {
                    // `$f:` is a fragment, not a keyword use.
                    let prev_dollar = k > 0 && matches!(&toks[k - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '$');
                    if !prev_dollar {
                        makes.insert(s.clone());
                    }
                }
                if s == self_name {
                    if let Some(proc_macro2::TokenTree::Punct(p)) = toks.get(k + 1) {
                        if p.as_char() == '!' {
                            *recursive = true;
                        }
                    }
                }
                if k >= 3 {
                    let is_frag = matches!(&toks[k - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == ':')
                        && matches!(&toks[k - 2], proc_macro2::TokenTree::Ident(_))
                        && matches!(&toks[k - 3], proc_macro2::TokenTree::Punct(p) if p.as_char() == '$');
                    if is_frag {
                        fragments.insert(s);
                    }
                }
            }
            proc_macro2::TokenTree::Group(gr) => {
                let prev_dollar = k > 0 && matches!(&toks[k - 1], proc_macro2::TokenTree::Punct(p) if p.as_char() == '$');
                if prev_dollar && gr.delimiter() == proc_macro2::Delimiter::Parenthesis {
                    *repetition = true;
                }
                scan_tokens(gr.stream(), self_name, makes, fragments, repetition, recursive);
            }
            _ => {}
        }
    }
}
