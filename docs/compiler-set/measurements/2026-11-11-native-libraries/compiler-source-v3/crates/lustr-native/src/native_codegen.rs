//! Native object emission and typed calls into independently compiled libraries.
//!
//! The checker, lowerer, borrow checker, C generator, and TinyCC backend are the
//! pinned upstream implementation. Imported declarations have no Rust body. The
//! one owned IR transformation loads a real, linker-initialized function pointer
//! and uses upstream's ordinary indirect-call ABI. A dependency's function body
//! is compiled once, into its library object; it is never stored as source here.

use crate::ast::{ItemKind, Vis};
use crate::check::collect::{Program, StaticItem};
use crate::check::generics::FnClass;
use crate::def::{DefId, DefKind, LocalDefId};
use crate::diag::{Diag, SourceMap};
use crate::expand::ClientView;
use crate::ir::{Body, CallTarget, LocalDecl, Place, Rvalue, Stmt, StmtKind, Terminator};
use crate::native_artifact::{AbiType, Export, Location, NativeArtifact};
use crate::native_library::LibraryProduct;
use crate::resolve::imports::{BindState, ResKind, Target, VisKind};
use crate::ty::defs::ModuleId;
use crate::ty::{Binder, FloatTy, FnSig, IntTy, Ty, TyIntern, TyKind};
use crate::{borrowck, cgen, check, driver, lower, tcc};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

static WORK_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct NativeError {
    pub code: i32,
    pub message: String,
}

impl std::fmt::Display for NativeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl From<String> for NativeError {
    fn from(message: String) -> Self {
        Self { code: 2, message }
    }
}

impl From<&str> for NativeError {
    fn from(message: &str) -> Self {
        Self::from(message.to_string())
    }
}

struct WorkDirectory(PathBuf);

impl WorkDirectory {
    fn create(parent: &Path) -> Result<Self, String> {
        fs::create_dir_all(parent).map_err(|error| format!("create native work parent: {error}"))?;
        for _ in 0..64 {
            let sequence = WORK_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("lustr-objects-{}-{sequence}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(format!("create private native object directory: {error}")),
            }
        }
        Err("could not create a unique native object directory".into())
    }
}

impl Drop for WorkDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn diagnostics(phase: &str, map: &SourceMap, errors: &[Diag]) -> NativeError {
    let mut out = format!("native library {phase} failed");
    for error in errors {
        let (path, line, column) = if (error.span.file.0 as usize) < map.file_count() {
            let (line, column) = map.line_col(error.span);
            (map.path(error.span.file), line, column)
        } else {
            ("<generated>", 0, 0)
        };
        out.push_str(&format!("\n{path}:{line}:{column}: {}: {}", error.code, error.message));
    }
    NativeError { code: 1, message: out }
}

fn materialize(program: &mut Program, map: &SourceMap) -> Result<(), NativeError> {
    program.gx.walk_instances = true;
    let errors = check::generics::materialize(program);
    if errors.is_empty() { Ok(()) } else { Err(diagnostics("instantiation", map, &errors)) }
}

fn abi_type(program: &Program, ty: Ty) -> Result<AbiType, String> {
    Ok(match program.tys.kind(ty) {
        TyKind::Unit => AbiType::Unit,
        TyKind::Bool => AbiType::Bool,
        TyKind::Char => AbiType::Char,
        TyKind::Int(IntTy::I8) => AbiType::I8,
        TyKind::Int(IntTy::I16) => AbiType::I16,
        TyKind::Int(IntTy::I32) => AbiType::I32,
        TyKind::Int(IntTy::I64) => AbiType::I64,
        TyKind::Int(IntTy::I128) => AbiType::I128,
        TyKind::Int(IntTy::Isize) => AbiType::Isize,
        TyKind::Int(IntTy::U8) => AbiType::U8,
        TyKind::Int(IntTy::U16) => AbiType::U16,
        TyKind::Int(IntTy::U32) => AbiType::U32,
        TyKind::Int(IntTy::U64) => AbiType::U64,
        TyKind::Int(IntTy::U128) => AbiType::U128,
        TyKind::Int(IntTy::Usize) => AbiType::Usize,
        TyKind::Float(FloatTy::F32) => AbiType::F32,
        TyKind::Float(FloatTy::F64) => AbiType::F64,
        _ => return Err("native library format v1 exports only primitive function parameters and results".into()),
    })
}

fn signature(program: &Program, function: LocalDefId) -> Result<(Vec<AbiType>, AbiType, bool), String> {
    let definition = program.fn_def(function);
    let name = program.display_path(program.fn_info(function).name);
    if definition.qual.is_const
        || definition.qual.is_async
        || definition.qual.is_unsafe
        || definition.qual.abi.is_some()
        || !definition.generics.params.is_empty()
        || !definition.where_clause.preds.is_empty()
        || definition.variadic
        || definition.self_param.is_some()
        || check::caller::is_track_caller(program, function)
    {
        return Err(format!(
            "native library format v1 cannot preserve the exported contract of `{name}` (const, async, unsafe, generic, method, extern, variadic, or track_caller)"
        ));
    }
    for attribute in &definition.attrs {
        // These affect documentation, code emitted in the producer, or its own
        // lint checks. Every other attribute needs an explicit metadata rule:
        // must_use/deprecated affect callers, and unsafe(no_mangle) changes
        // linkage even though the attribute's outer name is just `unsafe`.
        if !matches!(attribute.path.as_slice(), [part] if matches!(program.name(part.name),
            "doc" | "inline" | "cold" | "allow" | "warn" | "deny" | "forbid"))
        {
            return Err(format!("native library format v1 does not encode the exported attribute on `{name}`"));
        }
    }
    let info = program.fn_info(function);
    let params = info.params.iter().map(|&ty| abi_type(program, ty)).collect::<Result<Vec<_>, _>>()?;
    Ok((params, abi_type(program, info.ret)?, definition.qual.is_unsafe))
}

/// Enumerate the actual public resolver namespace. Private modules are not
/// traversed, while an explicit public re-export is represented by its visible
/// path. An unrepresented public item fails instead of disappearing from metadata.
fn exported_functions(program: &Program) -> Result<Vec<(Vec<String>, LocalDefId)>, String> {
    let mut result = Vec::new();
    if let Some(tables) = &program.resolved {
        let mut pending = vec![(ModuleId(0), Vec::<String>::new(), BTreeSet::<u32>::new())];
        while let Some((module, path, mut ancestors)) = pending.pop() {
            if !ancestors.insert(module.0) {
                return Err("native library format v1 cannot encode cyclic module re-exports".into());
            }
            for namespace in &tables.maps[module.0 as usize] {
                for (&name, state) in namespace {
                    let BindState::One(binding) = state else { continue };
                    if binding.vis != VisKind::Pub {
                        continue;
                    }
                    let mut path = path.clone();
                    path.push(program.name(name).to_string());
                    match binding.target {
                        Target::Module(child) => pending.push((child, path, ancestors.clone())),
                        Target::Def(definition) => {
                            let information = tables
                                .def_info
                                .get(&definition)
                                .ok_or_else(|| format!("native library has no metadata for public `{}`", path.join("::")))?;
                            if information.kind != ResKind::Fn {
                                return Err(format!(
                                    "native library format v1 cannot export `{}`: only free functions are represented",
                                    path.join("::")
                                ));
                            }
                            let function = program
                                .fn_names
                                .get(&information.flat)
                                .copied()
                                .ok_or_else(|| format!("native library cannot find public function `{}`", path.join("::")))?;
                            result.push((path, function));
                        }
                        _ => return Err(format!("native library format v1 cannot export `{}`", path.join("::"))),
                    }
                }
            }
        }
    } else {
        for item in &program.ast.items {
            if item.vis != Vis::Public || program.shipped_crate_at(item.span).is_some() {
                continue;
            }
            let ItemKind::Fn(definition) = &item.kind else {
                return Err("native library format v1 represents only public free functions".into());
            };
            let function = program
                .fn_names
                .get(&definition.name)
                .copied()
                .ok_or_else(|| "native library could not find its checked public function".to_string())?;
            result.push((vec![program.display_name(definition.name).to_string()], function));
        }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result.dedup();
    if result.is_empty() {
        return Err("native library format v1 requires at least one exported primitive function".into());
    }
    Ok(result)
}

fn stable_names(program: &Program, slots: &[u32], imports: &[(LocalDefId, Export)]) -> Result<Arc<cgen::names::Names>, String> {
    let mut instances = Vec::new();
    for (id, instance) in program.gx.adt_instance_of.iter().enumerate() {
        if let Some((definition, args)) = instance {
            let args = crate::ty::region_free_args_get(&program.tys, *args).unwrap_or(*args);
            let key = driver::graph::InstanceKey { def: *definition, args };
            instances.push((id as u32, program.instance_key_hash(&key).0));
        }
    }
    let mut names = driver::release_c::names(program, slots, slots.len(), &instances);
    let names_mut = Arc::get_mut(&mut names).ok_or_else(|| "native symbol names unexpectedly shared".to_string())?;
    for (function, export) in imports {
        let id = export
            .symbol
            .strip_prefix("f_")
            .filter(|id| id.len() == 16 && id.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()))
            .ok_or_else(|| "invalid imported native symbol".to_string())?;
        let slot = *slots.get(function.0 as usize).ok_or_else(|| "imported function is outside the checked program".to_string())?;
        names_mut.fns[slot as usize] = Some(id.to_string());
    }
    let mut seen = BTreeSet::new();
    for id in names_mut.fns.iter().flatten() {
        if !seen.insert(id.clone()) {
            return Err(format!("native function symbol collision for f_{id}"));
        }
    }
    Ok(names)
}

#[derive(Clone, Copy)]
struct ImportBinding {
    static_index: u32,
    pointer_type: Ty,
}

/// These are actual immutable data definitions, initialized by the native
/// backend to an address in the linked library. They have no Rust function body.
fn import_bindings(program: &mut Program, imports: &[(LocalDefId, Export)]) -> Result<BTreeMap<LocalDefId, ImportBinding>, String> {
    let mut bindings = BTreeMap::new();
    for (function, export) in imports {
        if bindings.contains_key(function) || (function.0 as usize) >= program.fns.len() {
            return Err("duplicate or invalid native imported function identity".into());
        }
        if program.fn_def(*function).body.is_some() {
            return Err("native imported declaration unexpectedly contains a source body".into());
        }
        let (params, result, is_unsafe) = signature(program, *function)?;
        if params != export.params || result != export.result || is_unsafe != export.is_unsafe {
            return Err(format!("checked signature does not match native artifact export `{}`", export.path.join("::")));
        }
        let info = program.fn_info(*function);
        let (params, result) = (info.params.clone(), info.ret);
        let inputs = program.tys.intern_tys(&params);
        let mut sig = FnSig::plain(inputs, result);
        sig.is_unsafe = is_unsafe;
        let sig = program.tys.intern_sig(Binder::plain(sig));
        let pointer_type = program.tys.intern(TyKind::FnPtr(sig));
        let name = format!("$lustr$import${}", export.symbol);
        let hash = crate::def::item_hash(program.stable_crate, &[], DefKind::Static, &name, 0);
        if hash.0 == 0 || program.static_defs.iter().any(|definition| definition.hash == hash) {
            return Err("native import-address definition identity collision".into());
        }
        let static_index = u32::try_from(program.statics.len()).map_err(|_| "too many native static definitions")?;
        program.statics.push(StaticItem {
            name: program.interner.intern(&name),
            ty: pointer_type,
            mutable: false,
            value: crate::ty::consteval::ConstValue::FnPtr(function.0),
            tls: None,
            value_fp: Default::default(),
        });
        program.static_defs.push(DefId { krate: program.krate, hash });
        bindings.insert(*function, ImportBinding { static_index, pointer_type });
    }
    Ok(bindings)
}

/// Preserve the checked call's arguments, result, continuation, and unwind edge.
/// Only its storage/linkage changes. The function pointer has the same full
/// primitive signature that the checker read from validated native metadata.
fn lower_imports(body: &mut Body<'_>, bindings: &BTreeMap<LocalDefId, ImportBinding>) -> Result<(), String> {
    for block in &mut body.blocks {
        for statement in &mut block.stmts {
            if let StmtKind::Assign(_, value) = &mut statement.kind
                && let Rvalue::FnValue(function, ty) = value
                && let Some(binding) = bindings.get(function)
                && matches!(body.tys.kind(*ty), TyKind::FnPtr(_))
            {
                if cgen::cty(&body.tys, *ty) != cgen::cty(&body.tys, binding.pointer_type) {
                    return Err("native imported function-pointer coercion has an incompatible signature".into());
                }
                *value = Rvalue::StaticLoad(binding.static_index);
            }
        }
        if let Terminator::Call { target: CallTarget::Fn(function), .. } = &block.term
            && let Some(binding) = bindings.get(function)
        {
            let local = u32::try_from(body.locals.len()).map_err(|_| "too many native IR locals")?;
            body.locals.push(LocalDecl { ty: binding.pointer_type, mutable: false, name: None });
            block.stmts.push(Stmt {
                kind: StmtKind::Assign(Place::local(local), Rvalue::StaticLoad(binding.static_index)),
                span: block.term_span,
            });
            if let Terminator::Call { target, .. } = &mut block.term {
                *target = CallTarget::Indirect(local);
            }
        }
    }
    Ok(())
}

struct Emitted {
    functions: Vec<cgen::FnOut>,
    slots: Vec<u32>,
    live: Vec<usize>,
    names: Arc<cgen::names::Names>,
    glue: cgen::GlueSet,
    locations: Vec<Location>,
}

fn emit_functions(
    program: &mut Program,
    map: &SourceMap,
    imports: &[(LocalDefId, Export)],
    display: &ClientView,
) -> Result<Emitted, NativeError> {
    materialize(program, map)?;
    let bindings = import_bindings(program, imports)?;
    let slots = (0..program.fns.len() as u32).collect::<Vec<_>>();
    let names = stable_names(program, &slots, imports)?;
    cgen::clear_extern_pool();
    let phase = cgen::layout_phase();
    let answer = cgen::in_layout_phase(phase, || {
        cgen::names::with(Some(&names), || -> Result<_, NativeError> {
            let mut functions: Vec<cgen::FnOut> = (0..program.fns.len()).map(|_| Default::default()).collect();
            let mut live = Vec::new();
            let mut locations = Vec::new();
            for index in 0..program.fns.len() {
                let function = LocalDefId(index as u32);
                if bindings.contains_key(&function) {
                    continue;
                }
                if !matches!(program.gx.fn_class.get(index), None | Some(FnClass::Plain | FnClass::Instance | FnClass::Closure)) {
                    continue;
                }
                let definition = program.fn_def(function);
                if definition.body.is_none() {
                    return Err(format!(
                        "native function `{}` has neither a body nor a validated library artifact",
                        program.display_path(definition.name)
                    )
                    .into());
                }
                let (checked, errors) = check::check_fn(program, function);
                if !errors.is_empty() {
                    return Err(diagnostics("body checking", map, &errors));
                }
                let mut body = lower::lower(program, function, checked);
                let errors = borrowck::borrowck(program, &body);
                if !errors.is_empty() {
                    return Err(diagnostics("borrow checking", map, &errors));
                }
                lower_imports(&mut body, &bindings)?;
                let line = map.line_col(definition.name_span).0;
                let emitted = cgen::gen_fn(program, &body, map, line, &slots);
                if let Some(error) = &emitted.error {
                    return Err(error.clone().into());
                }
                if !emitted.ll.is_empty() {
                    return Err("native library format v1 requires the native C/TinyCC backend".into());
                }
                if !emitted.has_code() {
                    return Err("native code generation returned no body for a checked function".into());
                }
                locations.push(Location {
                    symbol: names.fns[index].clone().ok_or_else(|| "native body lacks a stable symbol".to_string())?,
                    file: display.shown(map.path(definition.name_span.file)),
                    line,
                });
                functions[index] = emitted;
                live.push(index);
            }
            let mut glue = cgen::GlueSet::default();
            glue.add(program, &slots, functions.iter().flat_map(|function| function.glue.iter()));
            // Match upstream release-C placement. Each shard carries genuine glue;
            // a library that would need shared glue is explicitly rejected below.
            glue.shared = Some(glue.protos.keys().cloned().collect());
            Ok((functions, live, locations, glue))
        })
    });
    cgen::clear_extern_pool();
    let (functions, live, locations, glue) = answer?;
    Ok(Emitted { functions, slots, live, names, glue, locations })
}

fn compile_objects(work: &Path, sources: Vec<String>, debug: bool) -> Result<Vec<(String, Vec<u8>)>, String> {
    fs::create_dir_all(work).map_err(|error| format!("create native object directory: {error}"))?;
    let units: Vec<(PathBuf, String)> =
        sources.into_iter().enumerate().map(|(index, source)| (work.join(format!("unit{index:06}.o")), source)).collect();
    tcc::compile_units(&units, debug)?;
    units
        .iter()
        .enumerate()
        .map(|(index, (path, _))| {
            let bytes = fs::read(path).map_err(|error| format!("read native object {}: {error}", path.display()))?;
            Ok((format!("unit{index:06}.o"), bytes))
        })
        .collect()
}

pub fn emit_library(
    program: &mut Program,
    map: &SourceMap,
    work: &Path,
    debug: bool,
    display: &ClientView,
) -> Result<LibraryProduct, NativeError> {
    let public = exported_functions(program)?;
    for (_, function) in &public {
        signature(program, *function)?;
    }
    if program.statics.iter().enumerate().any(|(index, _)| program.static_live(index)) {
        return Err("native library format v1 does not yet encode per-crate static initialization".into());
    }
    let emitted = emit_functions(program, map, &[], display)?;
    if !emitted.glue.protos.is_empty() {
        return Err("native library format v1 does not yet encode cross-crate shared glue; primitive function bodies without generated glue are supported".into());
    }
    let mut exports = Vec::new();
    for (path, function) in public {
        let (params, result, is_unsafe) = signature(program, function)?;
        let symbol =
            emitted.names.fns[function.0 as usize].as_ref().ok_or_else(|| "public native function has no emitted symbol".to_string())?;
        exports.push(Export { path, symbol: format!("f_{symbol}"), params, result, is_unsafe });
    }
    let register = format!("lustr_register_{:016x}", program.stable_crate.0);
    let source = cgen::names::with(Some(&emitted.names), || {
        cgen::fn_unit(program, &emitted.functions, &emitted.live, &emitted.glue, &register, None)
    });
    let work = WorkDirectory::create(work)?;
    let objects = compile_objects(&work.0, vec![source], debug)?;
    Ok(LibraryProduct { exports, locations: emitted.locations, objects })
}

fn location_unit(locations: &[Location]) -> Result<String, String> {
    let mut unique = BTreeMap::<&str, &Location>::new();
    for location in locations {
        if let Some(previous) = unique.insert(&location.symbol, location)
            && previous != location
        {
            return Err(format!("native location metadata collision for {}", location.symbol));
        }
    }
    let entries: Vec<&Location> = unique.into_values().collect();
    let paths: Vec<&str> = entries.iter().map(|location| location.file.as_str()).collect();
    let files: Vec<u32> = (0..entries.len() as u32).collect();
    let lines: Vec<u32> = entries.iter().map(|location| location.line).collect();
    let names = Arc::new(cgen::names::Names {
        fns: entries.iter().map(|location| Some(location.symbol.clone())).collect(),
        adts: Vec::new(),
        by_id: Default::default(),
    });
    Ok(cgen::names::with(Some(&names), || cgen::names::loc_unit(&files, &cgen::SlotFiles { paths: &paths, of_slot: &files }, &lines, true)))
}

pub fn emit_binary(
    program: &mut Program,
    map: &SourceMap,
    work: &Path,
    out: &Path,
    debug: bool,
    imports: &[(LocalDefId, Export)],
    dependencies: &[NativeArtifact],
    display: &ClientView,
) -> Result<(), NativeError> {
    let emitted = emit_functions(program, map, imports, display)?;
    let main = program.main.ok_or_else(|| "native binary crate has no main function".to_string())?;
    let main_slot = emitted.slots[main.0 as usize];
    if !emitted.functions[main_slot as usize].has_code() {
        return Err("native main function has no generated body".into());
    }
    let register = format!("lustr_register_{:016x}", program.stable_crate.0);
    let mut locations = emitted.locations.clone();
    let mut seen_dependencies = BTreeSet::new();
    let mut dependency_objects = Vec::new();
    let work = WorkDirectory::create(work)?;
    for (index, dependency) in dependencies.iter().enumerate() {
        crate::native_artifact::validate(dependency)?;
        if !seen_dependencies.insert(dependency.disambiguator.clone()) {
            return Err("duplicate native dependency identity at link".into());
        }
        locations.extend(dependency.locations.iter().cloned());
        for (member_index, (_, bytes)) in dependency.objects.iter().enumerate() {
            let path = work.0.join(format!("dependency{index:06}_{member_index:06}.o"));
            fs::write(&path, bytes).map_err(|error| format!("materialize native dependency object: {error}"))?;
            dependency_objects.push(path);
        }
    }
    let mut sources = cgen::names::with(Some(&emitted.names), || {
        let mut units = vec![cgen::fn_unit(program, &emitted.functions, &emitted.live, &emitted.glue, &register, Some(main_slot))];
        units.extend(cgen::names::glue_shards(program, &emitted.glue, &emitted.functions, 16 * 1024));
        // This emits real imported pointer bindings through ConstValue::FnPtr,
        // using upstream's ABI and actual dependency body symbols.
        units.push(cgen::names::statics_unit(program, &emitted.glue));
        units.push(driver::release_c::table_unit(&[register]));
        units.push(cgen::main_unit(main_slot, &[], cgen::unwind_init(program, |function| emitted.slots[function.0 as usize])));
        units
    });
    sources.push(location_unit(&locations)?);
    let objects = compile_objects(&work.0, sources, debug)?;
    let mut paths: Vec<PathBuf> = objects.iter().map(|(name, _)| work.0.join(name)).collect();
    paths.extend(dependency_objects);
    paths.push(tcc::cached_runtime_object(&cgen::runtime_unit(), debug)?);
    // The user's previous executable remains intact if type checking, object
    // generation, or linking fails. Stage beside the destination so the final
    // rename is atomic even when the compiler's work directory is on another
    // filesystem.
    let parent = out.parent().ok_or_else(|| "native output has no parent directory".to_string())?;
    let stage = WorkDirectory::create(parent)?;
    let linked = stage.0.join("linked-output");
    tcc::link(&paths, &linked, debug)?;
    fs::rename(&linked, out).map_err(|error| NativeError::from(format!("publish native executable {}: {error}", out.display())))
}
