//! Native library boundary. The unchanged driver performs parsing, expansion,
//! resolution, type checking and borrow checking; its retained Program is the
//! input to Lustr's independently compiled artifact emitter. No Rust source is
//! stored as a substitute for object code, and no other compiler is invoked.

use crate::check::collect::Program;
use crate::native_artifact::{Export, Location, NativeArtifact, Profile};
use crate::protocol::{Cmd, Request, Response};
use std::path::Path;

pub struct LibraryProduct {
    pub exports: Vec<Export>,
    pub locations: Vec<Location>,
    pub objects: Vec<(String, Vec<u8>)>,
}

pub struct CompileOutput {
    pub response: Response,
    pub expand_deps: Vec<crate::expand::Dep>,
    pub unused_externs: Vec<String>,
}

impl From<Response> for CompileOutput {
    fn from(response: Response) -> Self {
        Self { response, expand_deps: Vec::new(), unused_externs: Vec::new() }
    }
}

/// Keep the driver's genuine checked Program instead of using the one-shot
/// process's usual arena leak. Check mode also validates private bodies. The
/// request's target kind distinguishes a library from a binary; no synthetic
/// main or alternate application source is introduced.
pub fn check(request: &Request, session: &mut crate::driver::Session) -> Result<(Program, Response), Response> {
    let mut check = request.clone();
    check.cmd = Cmd::Check;
    session.caches.leak_at_exit = false;
    let response = crate::driver::compile(&check, session);
    if response.code != 0 {
        return Err(response);
    }
    let Some(kept) = session.front.kept[0].take() else {
        return Err(Response {
            code: 2,
            stderr: "error: native checker did not retain its checked program\n".into(),
            ..Default::default()
        });
    };
    Ok((kept.program, response))
}

pub fn compiler_identity() -> String {
    format!("lustr-native-abi1:{}:{}", env!("CARGO_PKG_VERSION"), env!("LUST_BUILD_HASH"))
}

pub fn runtime_identity() -> String {
    // build.rs binds the entire pinned runtime source plus the linked TinyCC
    // libraries and executable into this hash. Separate compiler builds cannot
    // silently consume artifacts made against another runtime implementation.
    format!("lust-runtime-tinycc:{}", env!("LUST_BUILD_HASH"))
}

pub fn artifact_profile(request: &Request) -> Profile {
    Profile {
        panic_abort: request.session.profile.panic_abort,
        overflow_checks: request.session.profile.overflow_checks,
        debug_assertions: request.session.profile.debug_assertions,
        debug_info: request.debug,
    }
}

pub fn compile(request: &Request, session: &mut crate::driver::Session, metadata: Option<&Path>) -> CompileOutput {
    let (program, response, safety, expand_deps) = match check(request, session) {
        Ok((program, response)) => (program, response, crate::native_safety::NativeSafety::default(), Vec::new()),
        Err(original) => match crate::native_import::collect_extension(request, session) {
            Ok(Some(imported)) => (imported.program, Response::default(), imported.safety, imported.expand_deps),
            Ok(None) => return original.into(),
            Err(response) => return response.into(),
        },
    };
    let response = emit_checked_library(request, session, metadata, program, response, &safety);
    CompileOutput { response, expand_deps, unused_externs: Vec::new() }
}

fn emit_checked_library(
    request: &Request,
    session: &mut crate::driver::Session,
    metadata: Option<&Path>,
    mut program: Program,
    mut response: Response,
    safety: &crate::native_safety::NativeSafety,
) -> Response {
    let mut wrote_archive = false;
    let mut wrote_metadata = false;
    let result = (|| {
        let product =
            crate::native_codegen::emit_library(&mut program, &session.front.map, session.obj_dir(), request.debug, &session.front.client, safety)?;
        let artifact = NativeArtifact {
            crate_name: request.session.krate.target_name.clone(),
            disambiguator: format!("{:016x}", program.stable_crate.0),
            target: if request.session.target_triple.is_empty() {
                crate::protocol::host_triple().into()
            } else {
                request.session.target_triple.clone()
            },
            compiler: compiler_identity(),
            runtime: runtime_identity(),
            profile: artifact_profile(request),
            exports: product.exports,
            locations: product.locations,
            objects: product.objects,
        };
        crate::native_artifact::write(&request.out, &artifact)?;
        wrote_archive = true;
        if let Some(path) = metadata {
            crate::native_artifact::write_metadata(path, &artifact)?;
            wrote_metadata = true;
        }
        Ok::<(), crate::native_codegen::NativeError>(())
    })();
    if let Err(error) = result {
        if wrote_archive {
            let _ = std::fs::remove_file(&request.out);
        }
        if wrote_metadata {
            if let Some(path) = metadata {
                let _ = std::fs::remove_file(path);
            }
        }
        response.code = error.code;
        response.stderr.push_str(&format!("error: native library: {error}\n"));
    }
    response
}

pub fn compile_imported_binary(
    request: &Request,
    session: &mut crate::driver::Session,
    dependencies: Vec<(String, NativeArtifact)>,
) -> CompileOutput {
    let imported = match crate::native_import::collect(request, session, &dependencies) {
        Ok(imported) => imported,
        Err(response) => return response.into(),
    };
    let dependencies: Vec<NativeArtifact> = dependencies.into_iter().map(|(_, artifact)| artifact).collect();
    emit_imported_binary(request, session, imported, &dependencies)
}

/// The unchanged driver remains the first and only path for programs it
/// accepts. Owned syntax and semantic analysis supply explicit native extensions
/// after rejection, without executing another compiler or rewriting source.
pub fn compile_binary(request: &Request, session: &mut crate::driver::Session) -> CompileOutput {
    let original = crate::driver::compile(request, session);
    if original.code == 0 {
        return original.into();
    }
    match crate::native_import::collect_extension(request, session) {
        Ok(Some(imported)) => emit_imported_binary(request, session, imported, &[]),
        Ok(None) => original.into(),
        Err(response) => response.into(),
    }
}

fn emit_imported_binary(
    request: &Request,
    session: &mut crate::driver::Session,
    imported: crate::native_import::ImportedProgram,
    dependencies: &[NativeArtifact],
) -> CompileOutput {
    let crate::native_import::ImportedProgram { mut program, imports, expand_deps, unused_externs, safety } = imported;
    let result = crate::native_codegen::emit_binary(
        &mut program,
        &session.front.map,
        session.obj_dir(),
        &request.out,
        request.debug,
        &imports,
        dependencies,
        &session.front.client,
        &safety,
    );
    let response = match result {
        Ok(()) => Response::default(),
        Err(error) => Response { code: error.code, stderr: format!("error: native imported binary: {error}\n"), ..Default::default() },
    };
    CompileOutput { response, expand_deps, unused_externs }
}
