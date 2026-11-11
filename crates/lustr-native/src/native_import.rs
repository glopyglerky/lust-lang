//! Import typed native artifact interfaces into the unchanged checker.
//!
//! Imported functions are bodyless declarations constructed from validated
//! metadata. Consumer source is parsed unchanged. The original import resolver
//! and path rewriter bind its names to those declarations, and original type
//! checking/lowering/borrow checking handle the consumer. Object bodies are
//! never reconstructed from dependency source.

use crate::ast::*;
use crate::check::collect::{LocalDefId, Program};
use crate::def::{CrateIds, DefId};
use crate::diag::{Diag, FileId, SourceMap, Span};
use crate::driver::graph::Namespace;
use crate::intern::{FxMap, Interner, Sym};
use crate::native_artifact::{AbiType, Export, NativeArtifact};
use crate::protocol::{Request, Response};
use crate::resolve::imports::*;
use crate::ty::defs::{ModuleId, ROOT_MODULE};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub struct ImportedProgram {
    pub program: Program,
    pub imports: Vec<(LocalDefId, Export)>,
    pub expand_deps: Vec<crate::expand::Dep>,
    pub unused_externs: Vec<String>,
    pub safety: crate::native_safety::NativeSafety,
}

fn diagnostic(request: &Request, map: &SourceMap, text: &str, mut diags: Vec<Diag>) -> Response {
    let n = crate::diag::dedup_exact(&mut diags, map);
    Response { code: 1, stderr: crate::diag::render(&mut diags[..n], map, text, false, request.max_errors), ..Default::default() }
}

fn unsupported(message: impl Into<String>, span: Span) -> Diag {
    Diag::new("L1000", span, message, "This native artifact interface is not implemented yet.")
}

fn add_extern(tables: &mut ResolvedTables, ids: CrateIds, name: &str, root: Option<ModuleId>, standard: bool) {
    tables.externs.push(ExternInfo {
        name: name.into(),
        def: DefId { krate: ids.local, hash: crate::def::item_hash(ids.stable, &[], crate::def::DefKind::Module, name, 0) },
        is_std_lib: standard,
        owner: ROOT_MODULE,
        root_module: root,
        unsupported: None,
    });
}

fn bind_function(
    tables: &mut ResolvedTables,
    interner: &Interner,
    ids: CrateIds,
    module: ModuleId,
    name: Sym,
    flat: Sym,
    visibility: VisKind,
) -> DefId {
    let id = DefId { krate: ids.local, hash: crate::def::free_fn_hash(ids.stable, &[], interner.get(flat)) };
    tables.insert(module, Namespace::Value, name, Bind { target: Target::Def(id), vis: visibility.clone(), rank: 2, def_module: module });
    tables.def_info.insert(id, DefInfo { kind: ResKind::Fn, flat, module, vis: visibility });
    tables.flat[ns_index(Namespace::Value)].insert(flat, id);
    id
}

fn ast_type(ast: &mut Ast, ty: &AbiType, interner: &Interner, span: Span) -> TyId {
    let name = match ty {
        AbiType::Unit => None,
        AbiType::Bool => Some("bool"),
        AbiType::Char => Some("char"),
        AbiType::I8 => Some("i8"),
        AbiType::I16 => Some("i16"),
        AbiType::I32 => Some("i32"),
        AbiType::I64 => Some("i64"),
        AbiType::I128 => Some("i128"),
        AbiType::Isize => Some("isize"),
        AbiType::U8 => Some("u8"),
        AbiType::U16 => Some("u16"),
        AbiType::U32 => Some("u32"),
        AbiType::U64 => Some("u64"),
        AbiType::U128 => Some("u128"),
        AbiType::Usize => Some("usize"),
        AbiType::F32 => Some("f32"),
        AbiType::F64 => Some("f64"),
    };
    let id = TyId(ast.tys.len() as u32);
    ast.tys.push(name.map_or(TyExpr::Unit, |name| TyExpr::Path(vec![PathSeg::plain(interner.intern(name))])));
    ast.ty_spans.push(span);
    id
}

fn declaration(ast: &mut Ast, export: &Export, flat: Sym, interner: &Interner, span: Span) -> Item {
    let params = export
        .params
        .iter()
        .enumerate()
        .map(|(i, ty)| Param {
            name: interner.intern(&format!("argument_{i}")),
            mutable: false,
            pat: None,
            ty: ast_type(ast, ty, interner, span),
            attrs: Vec::new(),
            span,
        })
        .collect();
    let ret = Some(ast_type(ast, &export.result, interner, span));
    Item {
        vis: Vis::Public,
        attrs: Vec::new(),
        span,
        kind: ItemKind::Fn(FnDef {
            name: flat,
            name_span: span,
            is_test: false,
            vis: Vis::Public,
            attrs: Vec::new(),
            qual: FnQual { is_unsafe: export.is_unsafe, ..Default::default() },
            generics: Generics::default(),
            self_param: None,
            params,
            variadic: false,
            ret,
            where_clause: WhereClause::default(),
            body: None,
            exprs: (ast.exprs.len() as u32, ast.exprs.len() as u32),
            pats: (ast.pats.len() as u32, ast.pats.len() as u32),
        }),
    }
}

/// The first importer deliberately supports free functions and use declarations.
/// Unsupported semantic item forms are rejected after parsing, never guessed
/// from source words or silently replaced with another application.
pub fn collect(
    request: &Request,
    session: &mut crate::driver::Session,
    dependencies: &[(String, NativeArtifact)],
) -> Result<ImportedProgram, Response> {
    collect_impl(request, session, dependencies, false).map(|program| program.expect("unconditional native collection"))
}

/// An existing-driver rejection can enter the owned frontend only when its
/// parser has recognized a supported extension and retained its semantics.
/// Existing accepted programs stay on their original driver path.
pub fn collect_extension(
    request: &Request,
    session: &mut crate::driver::Session,
) -> Result<Option<ImportedProgram>, Response> {
    collect_impl(request, session, &[], true)
}

fn collect_impl(
    request: &Request,
    session: &mut crate::driver::Session,
    dependencies: &[(String, NativeArtifact)],
    extension_required: bool,
) -> Result<Option<ImportedProgram>, Response> {
    let text = std::fs::read_to_string(&request.file).map_err(|e| Response {
        code: 2,
        stderr: format!("error: read {}: {e}\n", request.file.display()),
        ..Default::default()
    })?;
    let mut map = SourceMap::default();
    map.add_file_with_text(&request.file.to_string_lossy(), &text);
    let err = |diags| diagnostic(request, &map, &text, diags);
    let interner = session.front.interner.clone();
    let known = session.front.known.clone();
    let ids = session.crate_ids();
    let env = crate::resolve::cfg::CfgEnv::new(&request.session.target_triple, &request.session.features, false, request.session.profile);
    let client = crate::expand::ClientView {
        env: request.client.env.clone().map(Arc::new),
        shown_root: request
            .client
            .shown
            .as_ref()
            .and_then(|typed| request.file.parent().zip(typed.parent()))
            .map(|(absolute, shown)| (absolute.to_path_buf(), shown.to_path_buf())),
    };
    let lexed = crate::lexer::lex(&text, FileId(0), 0, &interner).map_err(|d| err(vec![d]))?;
    crate::upstream::resolve::worklist::check_cfg_names(&lexed, &env, &interner).map_err(|d| err(vec![d]))?;
    let ctx = crate::expand::ExpandCtx {
        interner: &interner,
        known: &known,
        file_path: client.shown(&request.file.to_string_lossy()),
        real_path: request.file.clone(),
        cargo_env: Vec::new(),
        src: &text,
        cfg: crate::upstream::resolve::worklist::expand_cfg(&env),
        module_path: Vec::new(),
        crate_name: Some(request.session.krate.target_name.clone()),
        extern_crates: dependencies.iter().map(|(name, _)| interner.intern(name)).collect(),
        client: &client,
    };
    let (lexed, expand_deps) = crate::expand::expand_lexed_with_deps(lexed, &ctx).map_err(|d| err(vec![d]))?;
    let parsed = match crate::native_syntax::parse(&lexed, &interner, &known) {
        Ok(parsed) => parsed,
        Err(_) if extension_required => return Ok(None),
        Err(diagnostic) => return Err(err(vec![diagnostic])),
    };
    let crate::native_syntax::Parsed { mut ast, safety } = parsed;
    // The original collector treats a root function named `main` as a binary
    // entry point even for a library request. Native libraries have no entry
    // point; recognizing that parsed declaration is a separate target semantic.
    let library_main = request.session.krate.target_kind == "lib"
        && ast.items.iter().any(|item| matches!(&item.kind, ItemKind::Fn(def) if def.name == known.main));
    if extension_required && !safety.has_extension() && !library_main {
        return Ok(None);
    }
    if !ast.inner_attrs.is_empty() || !ast.stmt_items.is_empty() {
        return Err(err(vec![unsupported(
            "native artifact imports do not yet support crate attributes or body-local items",
            Span::default(),
        )]));
    }
    // Source std splicing and its platform crate must be composed through the
    // original driver's std planner before these forms can join imported crates.
    let asks = crate::stdload::StdPlan::asks(&text);
    if asks.needs.any() || asks.platform {
        return Err(err(vec![unsupported("native artifact imports do not yet compose source standard-library modules", Span::default())]));
    }
    let mut tables = ResolvedTables::new(interner.intern(&request.session.krate.target_name));
    tables.crate_cfgs.insert(ROOT_MODULE, (env, request.session.krate.target_name.clone()));
    tables.module_defs[0] = DefId {
        krate: ids.local,
        hash: crate::def::item_hash(ids.stable, &[], crate::def::DefKind::Module, &request.session.krate.target_name, 0),
    };
    for name in ["std", "core", "alloc"] {
        add_extern(&mut tables, ids, name, None, true);
    }
    let mut original_functions = Vec::new();
    let mut uses = Vec::new();
    let mut seen = BTreeSet::new();
    for item in std::mem::take(&mut ast.items) {
        match &item.kind {
            ItemKind::Fn(def) => {
                // Local generic helpers keep their original parameters and
                // bounds. The upstream collector and monomorphizer own their
                // type/borrow checks and instances; exported generic metadata
                // still requires its separate artifact-format implementation.
                if !seen.insert(def.name.0) {
                    return Err(err(vec![Diag::new(
                        "L0106",
                        def.name_span,
                        "duplicate function definition",
                        "Give each function a distinct name.",
                    )]));
                }
                bind_function(&mut tables, &interner, ids, ROOT_MODULE, def.name, def.name, vis_from_ast(&item.vis, &interner));
                original_functions.push(item);
            }
            ItemKind::Use(def) => flatten_use(&def.tree, ROOT_MODULE, vis_from_ast(&item.vis, &interner), &mut uses, &interner),
            ItemKind::MacroRules(_) => {} // Definitions were consumed by the real expander.
            _ => {
                return Err(err(vec![unsupported(
                    "native artifact imports currently support free functions and use declarations only",
                    item.span,
                )]));
            }
        }
    }
    let mut imported = Vec::new();
    let mut import_alias_by_flat = FxMap::default();
    let mut roots = BTreeMap::new();
    let mut identities = BTreeSet::new();
    for (alias, dependency) in dependencies {
        if !identities.insert((dependency.crate_name.clone(), dependency.disambiguator.clone())) {
            return Err(diagnostic(
                request,
                &map,
                &text,
                vec![unsupported("loading the same native crate under multiple aliases is not implemented yet", Span::default())],
            ));
        }
        let root = tables.add_crate_root(interner.intern(alias));
        tables.module_defs[root.0 as usize] =
            DefId { krate: ids.local, hash: crate::def::item_hash(ids.stable, &[], crate::def::DefKind::Module, alias, 0) };
        add_extern(&mut tables, ids, alias, Some(root), false);
        roots.insert(root.0, alias.clone());
        let file = map.add_file_with_text(&format!("<native metadata for {alias}>"), "");
        let span = Span { file, ..Span::default() };
        for export in &dependency.exports {
            let mut module = root;
            for segment in &export.path[..export.path.len() - 1] {
                let name = interner.intern(segment);
                module = match tables.lookup(module, Namespace::Type, name) {
                    Some(BindState::One(Bind { target: Target::Module(child), .. })) => *child,
                    _ => {
                        let child = tables.add_module(module, name, VisKind::Pub, span);
                        tables.insert(
                            module,
                            Namespace::Type,
                            name,
                            Bind { target: Target::Module(child), vis: VisKind::Pub, rank: 2, def_module: module },
                        );
                        child
                    }
                };
            }
            let name = interner.intern(export.path.last().unwrap());
            let flat = interner.intern(&format!("$lustr$external${}${}", dependency.disambiguator, export.symbol));
            bind_function(&mut tables, &interner, ids, module, name, flat, VisKind::Pub);
            tables.display_names.insert(flat, interner.intern(&format!("{alias}::{}", export.path.join("::"))));
            import_alias_by_flat.insert(flat, alias.clone());
            imported.push((flat, export.clone()));
            let item = declaration(&mut ast, export, flat, &interner, span);
            ast.items.push(item);
            tables.fn_module.push(module);
        }
    }
    let import_diagnostics =
        run_imports(&mut tables, &uses, &interner).map_err(|e| diagnostic(request, &map, &text, vec![unsupported(e, Span::default())]))?;
    if !import_diagnostics.is_empty() {
        return Err(diagnostic(request, &map, &text, import_diagnostics));
    }
    let local_names = tables.local_named_set();
    for mut item in original_functions {
        let ItemKind::Fn(ref mut def) = item.kind else { unreachable!() };
        let diagnostics =
            crate::upstream::resolve::worklist::rewrite_fn_item(&tables, &interner, &local_names, None, &mut ast, def, None, ROOT_MODULE);
        if !diagnostics.is_empty() {
            return Err(diagnostic(request, &map, &text, diagnostics));
        }
        ast.items.push(item);
        tables.fn_module.push(ROOT_MODULE);
    }
    let mut used = BTreeSet::new();
    for expression in &ast.exprs {
        if let Expr::Path(segments) = expression {
            if let [segment] = segments.as_slice() {
                if let Some(alias) = import_alias_by_flat.get(&segment.name) {
                    used.insert(alias.clone());
                }
            }
        }
    }
    // A resolved `use dep::item` consumes its crate even when the local binding
    // is later unused, matching the scope of rustc's unused-crate dependency set.
    for (_, _, source_module, _, _, _) in &tables.single_binds {
        if let Some(alias) = roots.get(&tables.root_of(*source_module).0) {
            used.insert(alias.clone());
        }
    }
    for (_, module) in tables.glob_src.iter() {
        if let Some(alias) = roots.get(&tables.root_of(module).0) {
            used.insert(alias.clone());
        }
    }
    // A successfully resolved empty group or glob still names its crate even
    // if there are no exported items to leave a copied binding in the tables.
    // Resolve the actual parsed import path; source spellings are not a proxy
    // for a dependency use, because aliases and shadowing can change it.
    for import in &uses {
        let path = match &import.kind {
            ImportKind::Single { path, .. } | ImportKind::Glob { path } | ImportKind::CheckPath { path } => path,
            _ => continue,
        };
        let root = match tables.resolve_module_path(import.module, path, import.leading_colon, &interner) {
            PathRes::Done(Target::Module(module)) => Some(tables.root_of(module)),
            PathRes::Done(Target::Extern(index)) => tables.externs[index as usize].root_module,
            _ => None,
        };
        if let Some(alias) = root.and_then(|root| roots.get(&root.0)) {
            used.insert(alias.clone());
        }
    }
    let ast = Arc::new(ast);
    session.front.map = map.clone();
    session.front.src = text.clone();
    session.front.ast = ast.clone();
    session.front.client = client;
    let collection_known = if request.session.krate.target_kind == "lib" {
        let mut names = crate::intern::Known::new(&interner);
        // Configure the binary-oriented collector for an artifact-only crate.
        // Function names, resolver bindings and exported symbols stay original.
        names.main = interner.intern("$lustr$library$no_entry");
        Arc::new(names)
    } else {
        known
    };
    let mut program = crate::check::collect::collect_with_order_resolved(
        ast,
        interner.clone(),
        collection_known,
        ids,
        false,
        &mut session.front.def_order,
        Some(Arc::new(tables)),
    )
    .map_err(|diags| diagnostic(request, &map, &text, diags))?;
    program.unwind = !request.session.profile.panic_abort;
    program.overflow_checks = request.session.profile.overflow_checks;
    program.debug_assertions = request.session.profile.debug_assertions;
    program.unwind_tables = false;
    if request.session.krate.target_kind != "lib" && program.main.is_none() {
        return Err(diagnostic(
            request,
            &map,
            &text,
            vec![Diag::new("L0130", Span::default(), "no main function", "Define the binary crate entry point.")],
        ));
    }
    if request.session.krate.target_kind != "lib"
        && let Some(main) = program.main
        && program.fn_def(main).qual.is_unsafe
    {
        return Err(diagnostic(
            request,
            &map,
            &text,
            vec![Diag::new(
                "L1001",
                program.fn_def(main).name_span,
                "the main function must be safe",
                "Keep the entry point safe and put individual unsafe operations in explicit unsafe blocks.",
            )],
        ));
    }
    let imports = imported
        .into_iter()
        .map(|(flat, export)| {
            let id = program.fn_names.get(&flat).copied().ok_or_else(|| {
                diagnostic(request, &map, &text, vec![unsupported("collected native import is missing", Span::default())])
            })?;
            Ok((id, export))
        })
        .collect::<Result<Vec<_>, Response>>()?;
    // The emitter runs the original checker, lowering and borrow checker once
    // over every real consumer body before adapting its checked external calls.
    // Imported declarations have no bodies and are checked by their producers.
    Ok(Some(ImportedProgram {
        program,
        imports,
        expand_deps,
        unused_externs: dependencies.iter().filter(|(alias, _)| !used.contains(alias)).map(|(alias, _)| alias.clone()).collect(),
        safety,
    }))
}
