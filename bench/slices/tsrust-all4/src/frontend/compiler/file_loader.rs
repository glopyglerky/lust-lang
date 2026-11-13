//! Go: compiler/fileloader.go (the file loader that finds, parses and
//! resolves every program file).
//!
//! PORT: Go `opts.Tracing` is the process tracing session
//! (`crate::tracing::get`).

use super::files_parser::{note_prep_taken};
use crate::contentmapper::{
    DiagnosticDirectiveError, DiagnosticDirectiveErrorKind, InitializeError,
    InitializeErrorKind, InvalidVirtualExtensionError, Mapper, ProjectError, ProjectErrorKind,
    SupplementalFileCollisionError, TransformError, TransformErrorKind,
};
use crate::frontend::prelude::*;
use crate::gostd::{GoError, errors};
use crate::spanmap::{MappingError, MappingErrorKind};
use std::cell::Cell;
use std::sync::Arc;

// Go: fileloader.go:34 maxContentMapperFailures (tsgo#4712)
// maxContentMapperFailures is the number of transform failures a single content mapper may accumulate
// before it is disabled for the rest of the program.
const MAX_CONTENT_MAPPER_FAILURES: i32 = 5;

// Go: fileloader.go:26 libResolution

// Go: fileloader.go:36 LibFile
#[derive(Clone, Debug, Default)]
pub struct LibFile {
    pub name: String,
    pub path: String,
    pub replaced: bool,
}

// Go: fileloader.go:42 sourceFileFromReferenceDiagnostic

// Go: fileloader.go:47 fileLoader
// PORT: the Go loader is shared by the parse work group. The port is single
// threaded (contract 10), so the atomics and sync maps are `Cell` and
// `RefCell`, and `factoryMu` is not needed. `resolver` is `None` only until
// `process_all_program_files` sets it, as in Go.

// Go: fileloader.go:81 redirectsFile
#[derive(Clone, Debug, Default)]
pub struct RedirectsFile {
    // Index of file at which this redirect file needs to be iterated
    pub index: i32,
    pub file_name: String,
    pub path: Path,
    pub target: Path,
}

// Go: fileloader.go:89 DuplicateSourceFile
// PORT: Go keeps `Hash xxh3.Uint128` for the language server parse cache.
// Here `hash` is the hash that a parse cache set on the file
// (`ParsedSourceFile::hash`), and `text` is kept so the language server can
// hash it when none was set (`source_hash`), as Go `fh.Hash()`.


impl RedirectsFile {
    // Go: fileloader.go:105 (*redirectsFile).FileName
    pub fn file_name(&self) -> String {
        self.file_name.clone()
    }

    // Go: fileloader.go:109 (*redirectsFile).Path
    pub fn path(&self) -> Path {
        self.path.clone()
    }
}

// Go: fileloader.go:113 processedFiles
// PORT: Go nil maps that stay nil until first use are `Option`. Go
// `*includeProcessor` is owned by value. Go `UpdateProgram` copies this
// struct and so shares its maps with the old program. The maps that stay
// the same after loading are behind `Rc`, so a clone shares them too; a
// deep copy would cost time and memory once per edit. The module
// resolutions are behind `Arc`, because checker threads read the same map
// (`GoSharedState`) with no copy.

// Go: fileloader.go:147 jsxRuntimeImportSpecifier
#[derive(Clone, Debug)]
pub struct JsxRuntimeImportSpecifier {
    pub module_reference: String,
    pub specifier: Node,
}

// Go: fileloader.go:152 processAllProgramFiles

#[cfg(test)]
thread_local! {
    /// Each worker prep that the loads on this thread looked at: the file
    /// name and whether the loader could use it (`FilePrep::fits`).
    static PREP_FITS: std::cell::RefCell<Vec<(String, bool)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {
    /// Set while `with_loader_state_forgotten` runs.
    static FORGET_LOADER_STATE: Cell<bool> = const { Cell::new(false) };
}


// Go: fileloader.go:467 contentMapperTransformDiagnostic (tsgo#4712)
// PORT: Go `*ast.SourceFile` is the file's SourceFile node. Go
// `errors.AsType` on a found `*TransformError` searches that error and the
// errors it wraps (`TransformError::to_go_error`).

// Go: fileloader.go:523 ContentMapperProjectErrorDiagnostic (tsgo#4712)
// ContentMapperProjectErrorDiagnostic returns the localized diagnostic message for a project setup error.

// Go: fileloader.go:541 contentMapperTransformDiagnosticChain (tsgo#4712)

// Go: fileloader.go:545 contentMapperTransformDiagnosticWithDetail (tsgo#4712)

// Go: fileloader.go:556 contentMapperMappingDiagnostic (tsgo#4712)
// contentMapperMappingDiagnostic builds the diagnostic reported against a mapper that produced an
// invalid span map, including the offsets involved so the mapper's author can locate the problem.

// Go: fileloader.go:601 ContentMapperInitializationDiagnostic (tsgo#4712)
// ContentMapperInitializationDiagnostic returns a fileless diagnostic for a mapper initialization failure.

// Go: fileloader.go:630 ContentMapperProjectDiagnostic (tsgo#4712)
// ContentMapperProjectDiagnostic returns a fileless diagnostic for project setup or mapper initialization.


/// Whether Go `loadSourceFileMetaData` takes the package.json `type` of the
/// scope of `file_name` (fileloader.go:398 to :401): for node16 to nodenext
/// module resolution (not for `.mts`, `.cts`, `.mjs` and `.cjs` files), or
/// for a path under `/node_modules/`.
pub(crate) fn package_json_type_applies(
    file_name: &str,
    module_resolution_kind: ModuleResolutionKind,
) -> bool {
    !file_extension_is_one_of(
        file_name,
        &[EXTENSION_MTS, EXTENSION_CTS, EXTENSION_MJS, EXTENSION_CJS],
    ) && ModuleResolutionKind::NODE16 <= module_resolution_kind
        && module_resolution_kind <= ModuleResolutionKind::NODE_NEXT
        || file_name.contains("/node_modules/")
}




// Go: fileloader.go:1000 getLibraryNameFromLibFileName
pub fn get_library_name_from_lib_file_name(lib_file_name: &str) -> String {
    // Support resolving to lib.dom.d.ts -> @typescript/lib-dom, and
    //                      lib.dom.iterable.d.ts -> @typescript/lib-dom/iterable
    //                      lib.es2015.symbol.wellknown.d.ts -> @typescript/lib-es2015/symbol-wellknown
    let components: Vec<&str> = lib_file_name.split('.').collect();
    let mut path = String::from("@typescript/lib-");
    if components.len() > 1 {
        path.push_str(components[1]);
    }
    let mut i = 2;
    while i < components.len() && !components[i].is_empty() && components[i] != "d" {
        if i == 2 {
            path.push('/');
        } else {
            path.push('-');
        }
        path.push_str(components[i]);
        i += 1;
    }
    path
}

// Go: fileloader.go:1023 getInferredLibraryNameResolveFrom

// Go: fileloader.go:1033 getModeForTypeReferenceDirectiveInFile

// Go: fileloader.go:1041 getDefaultResolutionModeForFile
// PORT: private, because program.rs has a public
// `get_default_resolution_mode_for_file` (Go program.go) with another shape.

// Go: fileloader.go:1049 getModeForUsageLocation
// PORT: private, because program.rs has a public `get_mode_for_usage_location`
// (Go program.go) with another shape. Go `options` can be nil (`None`). The
// node reads are `import_usage`, and the rest is `mode_for_import_usage`,
// so the loader can keep the node reads of a parse (`import_names`).

/// What Go `getModeForUsageLocation` reads of a module name node: a
/// resolution-mode override, or the kind of usage that
/// `getEmitSyntaxForUsageLocationWorker` tells apart (`import_usage`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ImportUsage {
    /// The `resolution-mode` of a type-only import or export, or of an
    /// import type.
    Override(ResolutionMode),
    /// `require(...)`, or `import x = require(...)`.
    Require,
    /// `import(...)`.
    ImportCall,
    /// Any other module name.
    Other,
}




/// The module names of the imports and string module augmentations of a
/// parse, in the order of `resolveImportsAndModuleAugmentations` after its
/// synthetic imports, with each name's `ImportUsage` and, for an import,
/// `import_adds_file`.
// PORT: not in Go (perf, `import_names`).
struct ImportNames {
    /// The names, one after another.
    text: String,
    /// Per name: its end in `text`, its usage, and whether it adds its file
    /// (false for a module augmentation, which adds none).
    names: Vec<(usize, ImportUsage, bool)>,
}

impl ImportNames {


    fn len(&self) -> usize {
        self.names.len()
    }

    /// The name, usage and `import_adds_file` of name `index`.
    fn get(&self, index: usize) -> (&str, ImportUsage, Option<bool>) {
        let (end, usage, adds) = self.names[index];
        let start = index
            .checked_sub(1)
            .map_or(0, |before| self.names[before].0);
        (&self.text[start..end], usage, Some(adds))
    }
}




// Go: fileloader.go:1081 importSyntaxAffectsModuleResolution

// Go: fileloader.go:1087 getEmitSyntaxForUsageLocationWorker



/// The synthetic imports of a file (Go `resolveImportsAndModuleAugmentations`,
/// before its imports): `tslib` for `importHelpers`, then the JSX runtime
/// import. The parse worker of the file resolves them too (`FilePrep`).
// Go: fileloader.go:844 to :862 (resolveImportsAndModuleAugmentations)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SyntheticImports {
    /// The file gets the `tslib` import (`EXTERNAL_HELPERS_MODULE_NAME_TEXT`).
    pub helpers: bool,
    /// The JSX runtime import, or empty.
    pub jsx: String,
}

impl SyntheticImports {

}


#[cfg(test)]
mod tests {
    use super::super::files_parser::{preps_taken, set_load_prep, set_meta_wait};
    use super::*;
    use crate::frontend::bundled;
    use crate::frontend::module::cache::set_answer_wait;
    use crate::frontend::tsoptions::{ParseConfigHost, get_parsed_command_line_of_config_file};
    use crate::frontend::vfs::osvfs_fs;

    struct System {
        fs: Rc<dyn Fs>,
        current_directory: String,
    }

    impl ParseConfigHost for System {
        fn fs(&self) -> Rc<dyn Fs> {
            self.fs.clone()
        }
        fn get_current_directory(&self) -> String {
            self.current_directory.clone()
        }
    }

    /// Writes `files` (path, text) and a tsconfig.json with `tsconfig` to a
    /// temp dir, loads the program and returns its file names in program
    /// order: paths relative to the dir, or the base name for a lib file.
    fn program_file_names(label: &str, tsconfig: &str, files: &[(&str, &str)]) -> Vec<String> {
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_file_loader_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::write(dir.join("tsconfig.json"), tsconfig).unwrap();
        let cwd = dir.to_string_lossy().replace('\\', "/");
        let fs = bundled::wrap_fs(osvfs_fs());
        let sys = System {
            fs: fs.clone(),
            current_directory: cwd.clone(),
        };
        let (config, errors) = get_parsed_command_line_of_config_file(
            &format!("{cwd}/tsconfig.json"),
            None,
            None,
            &sys,
            None,
        );
        assert!(errors.is_empty());
        let host = new_cached_fs_compiler_host(&cwd, fs, &bundled::lib_path(), None, None, None);
        let processed = process_all_program_files(
            ProgramOptions {
                host,
                config: Rc::new(config.unwrap()),
                use_source_of_project_reference: false,
                single_threaded: Tristate::True,
                typings_location: String::new(),
                project_name: String::new(),
                create_module_resolver: None,
                skip_module_resolution: false,
            },
            true,
        );
        let _ = std::fs::remove_dir_all(&dir);
        let prefix = format!("{cwd}/");
        processed
            .files
            .iter()
            .map(|file| match file.file_name().strip_prefix(&prefix) {
                Some(relative) => relative.to_string(),
                None => get_base_file_name(file.file_name()),
            })
            .collect()
    }

    /// Writes `files` and a tsconfig.json with `tsconfig` to a temp dir,
    /// loads the program with parse workers (not single threaded), and
    /// returns what the load found (`load_text`).
    fn parallel_load(label: &str, tsconfig: &str, files: &[(&str, &str)]) -> String {
        load_text(label, tsconfig, files, false)
    }

    /// Writes `files` and a tsconfig.json with `tsconfig` to a temp dir,
    /// loads the program, and returns what the load found, as text: the
    /// files in program order, the metadata, module and type reference
    /// resolutions (in map order) and include reasons of each file, the
    /// synthetic imports, the processing diagnostics, the missing files
    /// and the package.json entries of the resolver.
    fn load_text(
        label: &str,
        tsconfig: &str,
        files: &[(&str, &str)],
        single_threaded: bool,
    ) -> String {
        use std::fmt::Write as _;
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_file_loader_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::write(dir.join("tsconfig.json"), tsconfig).unwrap();
        let cwd = dir.to_string_lossy().replace('\\', "/");
        // Taken while the dir exists: the real path of a removed dir is the
        // path itself.
        let real = osvfs_fs().realpath(&cwd);
        let fs = bundled::wrap_fs(osvfs_fs());
        let sys = System {
            fs: fs.clone(),
            current_directory: cwd.clone(),
        };
        let (config, errors) = get_parsed_command_line_of_config_file(
            &format!("{cwd}/tsconfig.json"),
            None,
            None,
            &sys,
            None,
        );
        assert!(errors.is_empty());
        let host = new_cached_fs_compiler_host(&cwd, fs, &bundled::lib_path(), None, None, None);
        let processed = process_all_program_files(
            ProgramOptions {
                host,
                config: Rc::new(config.unwrap()),
                use_source_of_project_reference: false,
                single_threaded: if single_threaded {
                    Tristate::True
                } else {
                    Tristate::False
                },
                typings_location: String::new(),
                project_name: String::new(),
                create_module_resolver: None,
                skip_module_resolution: false,
            },
            single_threaded,
        );
        let _ = std::fs::remove_dir_all(&dir);
        let mut out = String::new();
        // The include reasons name files by path (Go `tspath.Path`), which
        // is in lower case on a case-insensitive file system (macOS).
        let case_sensitive = osvfs_fs().use_case_sensitive_file_names();
        let cwd_path = to_path(&cwd, "", case_sensitive).0;
        // A node_modules file is named by its real path (Go
        // `resolutionState.realPath`, module/resolver.go:1859). On macOS the
        // temp dir is under /var, a symlink to /private/var, and the real
        // path contains the plain one, so it is replaced first.
        let real_path = to_path(&real, "", case_sensitive).0;
        let name = |file_name: &str| {
            file_name
                .replace(&real, "<dir>")
                .replace(&real_path, "<dir>")
                .replace(&cwd, "<dir>")
                .replace(&cwd_path, "<dir>")
        };
        for file in &processed.files {
            let path = file.path();
            writeln!(out, "file {}", name(file.file_name())).unwrap();
            let meta = format!("{:?}", processed.source_file_meta_datas.get(path));
            writeln!(out, "  meta {}", name(&meta)).unwrap();
            for (key, resolved) in processed.resolved_modules.get(path).into_iter().flatten() {
                writeln!(
                    out,
                    "  module {:?} {:?} -> {} {:?} external {} package {}",
                    key.name,
                    key.mode,
                    name(&resolved.resolved_file_name),
                    resolved.extension,
                    resolved.is_external_library_import,
                    resolved.package_id.string(),
                )
                .unwrap();
            }
            for (key, resolved) in processed
                .type_resolutions_in_file
                .get(path)
                .into_iter()
                .flatten()
            {
                writeln!(
                    out,
                    "  type {:?} {:?} -> {}",
                    key.name,
                    key.mode,
                    name(&resolved.resolved_file_name)
                )
                .unwrap();
            }
            for reason in processed
                .include_processor
                .file_include_reasons
                .get(path)
                .into_iter()
                .flatten()
            {
                let data = match &reason.data {
                    FileIncludeData::ReferencedFile(data) => format!(
                        "{} #{} synthetic {}",
                        name(&data.file.0),
                        data.index,
                        data.synthetic.is_some()
                    ),
                    other => format!("{other:?}"),
                };
                writeln!(out, "  reason {:?} {data}", reason.kind).unwrap();
            }
            let jsx = processed
                .jsx_runtime_import_specifiers
                .as_ref()
                .and_then(|map| map.get(path))
                .map(|jsx| jsx.module_reference.clone());
            let helpers = processed
                .import_helpers_import_specifiers
                .as_ref()
                .is_some_and(|map| map.contains_key(path));
            writeln!(out, "  jsx {jsx:?} helpers {helpers}").unwrap();
        }
        for diagnostic in &processed.include_processor.processing_diagnostics {
            writeln!(out, "processing diagnostic {:?}", diagnostic.kind.0).unwrap();
        }
        writeln!(out, "missing {:?}", processed.missing_files).unwrap();
        let mut package_jsons = Vec::new();
        if let Some(resolver) = processed
            .resolver
            .as_ref()
            .and_then(|resolver| resolver.as_default_resolver())
        {
            resolver.package_json_cache_entries(|_, entry| {
                package_jsons.push(format!(
                    "{} {} {}",
                    name(entry.package_directory),
                    entry.directory_exists,
                    entry.exists
                ));
                true
            });
        }
        package_jsons.sort();
        package_jsons.dedup();
        writeln!(out, "package.json {package_jsons:?}").unwrap();
        out
    }

    // loadpar1: the parse workers resolve each file's synthetic imports and
    // imports for the loader (`FilePrep`). A load that takes their answers
    // finds what a load that resolves every name itself finds, in the same
    // order: node16 modes from package.json `type`, `.cts` and `.mts`,
    // `require`, `import()` (another mode than the worker's), type-only
    // imports with `resolution-mode`, JSDoc imports in TS and JS files,
    // `importHelpers`, the JSX runtime import, `/// <reference types>` and
    // a module augmentation.
    #[test]
    fn worker_prep_gives_the_same_load() {
        let tsconfig = r#"{ "compilerOptions": { "module": "node16", "allowJs": true,
             "checkJs": true, "importHelpers": true, "jsx": "react-jsx", "types": [],
             "noEmit": true }, "include": ["src"] }"#;
        let files = [
            ("package.json", r#"{ "name": "app", "type": "commonjs" }"#),
            ("src/esm/package.json", r#"{ "type": "module" }"#),
            (
                "src/esm/a.ts",
                r#"import { b } from "../b.js";
import type { T } from "lib" with { "resolution-mode": "require" };
import { e } from "lib";
export const a = b + e;
export type U = T;
export const later = () => import("../c.cjs");
"#,
            ),
            (
                "src/b.ts",
                r#"/// <reference types="node-ish" />
import x = require("./c.cjs");
import { e } from "lib";
export const b: number = x.c + e;
export const lazy = () => import("lib");
declare module "lib" { export const extra: number; }
"#,
            ),
            ("src/c.cts", "export const c = 1;\n"),
            (
                "src/d.mts",
                "import { e } from \"lib\";\nexport const d = e;\n",
            ),
            (
                "src/j.js",
                r#"/** @import { T } from "lib" */
const { c } = require("./c.cjs");
/** @type {T} */
export const j = c;
"#,
            ),
            (
                "src/k.ts",
                "/** @import { T } from \"./c.cjs\" */\nexport const k = 1;\n",
            ),
            ("src/v.tsx", "export const v = <div />;\n"),
            (
                "node_modules/lib/package.json",
                r#"{ "name": "lib", "version": "1.0.0",
                   "exports": { ".": { "import": "./esm.d.mts", "require": "./cjs.d.cts" } } }"#,
            ),
            (
                "node_modules/lib/esm.d.mts",
                "export declare const e: number;\nexport type T = string;\n",
            ),
            (
                "node_modules/lib/cjs.d.cts",
                "export declare const e: number;\nexport type T = number;\n",
            ),
            (
                "node_modules/tslib/package.json",
                r#"{ "name": "tslib", "version": "2.0.0", "types": "tslib.d.ts" }"#,
            ),
            (
                "node_modules/tslib/tslib.d.ts",
                "export declare function __assign(t: any): any;\n",
            ),
            (
                "node_modules/react/package.json",
                r#"{ "name": "react", "version": "18.0.0",
                   "exports": { "./jsx-runtime": { "types": "./jsx-runtime.d.ts" } } }"#,
            ),
            (
                "node_modules/react/jsx-runtime.d.ts",
                r#"export declare function jsx(): any;
export declare namespace JSX { interface IntrinsicElements { [name: string]: any } }
"#,
            ),
            (
                "node_modules/@types/node-ish/package.json",
                r#"{ "name": "@types/node-ish", "version": "1.0.0", "types": "index.d.ts" }"#,
            ),
            (
                "node_modules/@types/node-ish/index.d.ts",
                "declare var nodeIsh: number;\n",
            ),
        ];
        set_load_prep(Some(true));
        let before = preps_taken();
        let with_prep = parallel_load("prep_on", tsconfig, &files);
        let taken = preps_taken() - before;
        set_load_prep(Some(false));
        let without_prep = parallel_load("prep_off", tsconfig, &files);
        set_load_prep(None);
        assert_eq!(with_prep, without_prep);
        assert!(with_prep.contains(r#"module "lib""#), "{with_prep}");
        // With no parse workers (one CPU, or `GOPORT_PARSE_THREADS=0`) the
        // loader resolves every name itself.
        if super::super::files_parser::parse_workers_enabled() {
            assert!(taken > 0, "no prep taken:\n{with_prep}");
        }
    }

    /// A program with a project reference (`lib`) that it loads by the
    /// output `.d.ts` files (Go `getParseFileRedirect`, a program that does
    /// not use the sources of its references). The names in `lib/out` and
    /// in `lib/src/types.d.ts` (a source with no output) resolve with the
    /// options of `lib/tsconfig.json`, from their source files (Go
    /// `getRedirectForResolution`): its `paths` find `@lib/helper`, and its
    /// custom condition finds `dep/lib.d.ts`. `src/a.ts` resolves `dep`
    /// with the root options, to `dep/index.d.ts`.
    const REFERENCE_TSCONFIG: &str = r#"{ "compilerOptions": { "module": "esnext",
         "moduleResolution": "bundler", "types": [], "noEmit": true },
         "include": ["src"], "references": [{ "path": "./lib" }] }"#;
    const REFERENCE_FILES: [(&str, &str); 10] = [
        (
            "lib/tsconfig.json",
            r#"{ "compilerOptions": { "composite": true, "module": "esnext",
                 "moduleResolution": "bundler", "types": [], "rootDir": "src",
                 "outDir": "out", "paths": { "@lib/*": ["./src/*"] },
                 "customConditions": ["lib"] }, "include": ["src"] }"#,
        ),
        (
            "lib/src/index.ts",
            "export { h } from \"@lib/helper\";\nexport const x = 1;\n",
        ),
        ("lib/src/helper.ts", "export const h = 2;\n"),
        (
            "lib/src/types.d.ts",
            "import type { Dep } from \"dep\";\nimport type { h } from \"@lib/helper\";\nexport type T = Dep | typeof h;\n",
        ),
        (
            "lib/out/index.d.ts",
            "export { h } from \"@lib/helper\";\nexport declare const x = 1;\n",
        ),
        (
            "lib/out/helper.d.ts",
            "import type { Dep } from \"dep\";\nexport declare const h: Dep;\n",
        ),
        (
            "node_modules/dep/package.json",
            r#"{ "name": "dep", "version": "1.0.0",
               "exports": { ".": { "lib": "./lib.d.ts", "types": "./index.d.ts" } } }"#,
        ),
        ("node_modules/dep/lib.d.ts", "export type Dep = \"lib\";\n"),
        (
            "node_modules/dep/index.d.ts",
            "export type Dep = \"root\";\n",
        ),
        (
            "src/a.ts",
            r#"import { x, h } from "../lib/src/index";
import type { T } from "../lib/src/types";
import type { Dep } from "dep";
export const a: T | Dep | number = x + (h as never);
"#,
        ),
    ];

    // loadcrit2: with project references, the parse workers resolve each
    // file of a reference with the reference's options, from its source
    // file, as the loader does (Go `getRedirectForResolution`,
    // fileloader.go:842): the output `.d.ts` files and a `.d.ts` source.
    // The loader takes their preps (`FilePrep::fits`), and the load finds
    // what a load that resolves every name itself finds, and what a single
    // threaded load finds.
    #[test]
    fn workers_resolve_with_the_reference_redirect_of_the_loader() {
        let load = |label: &str, prep: bool, single_threaded: bool| {
            set_load_prep(Some(prep));
            set_meta_wait(prep && !single_threaded);
            let text = load_text(label, REFERENCE_TSCONFIG, &REFERENCE_FILES, single_threaded);
            set_load_prep(None);
            set_meta_wait(false);
            text
        };
        let single = load("refs_st", false, true);
        assert_eq!(load("refs_prep_off", false, false), single);
        for name in [
            "lib/out/index.d.ts",
            "lib/out/helper.d.ts",
            "lib/src/types.d.ts",
        ] {
            assert!(single.contains(&format!("file <dir>/{name}")), "{single}");
        }
        for target in ["dep/lib.d.ts", "dep/index.d.ts"] {
            assert!(
                single.contains(&format!("-> <dir>/node_modules/{target} ")),
                "{single}"
            );
        }
        // A worker publishes its prep after the parse that the loader
        // waits for, so the loader may resolve a file itself before it.
        // Each redirected file's prep fits in some load, and no prep is
        // unfit.
        let redirected = [
            "/lib/out/index.d.ts",
            "/lib/out/helper.d.ts",
            "/lib/src/types.d.ts",
        ];
        let mut fitted = [false; 3];
        for run in 0..20 {
            PREP_FITS.with(|fits| fits.borrow_mut().clear());
            assert_eq!(load(&format!("refs_prep_on_{run}"), true, false), single);
            let fits = PREP_FITS.with(|fits| std::mem::take(&mut *fits.borrow_mut()));
            assert!(fits.iter().all(|(_, fit)| *fit), "unfit preps: {fits:?}");
            for (index, name) in redirected.iter().enumerate() {
                fitted[index] |= fits.iter().any(|(file, _)| file.ends_with(name));
            }
            if !super::super::files_parser::parse_workers_enabled() || fitted.iter().all(|f| *f) {
                break;
            }
        }
        if super::super::files_parser::parse_workers_enabled() {
            assert_eq!(fitted, [true; 3], "{redirected:?}");
        }
    }

    /// Loads a one-file project with `compilerOptions` and returns the base
    /// names of the lib files in the program.
    fn lib_file_names(label: &str, compiler_options: &str) -> Vec<String> {
        program_file_names(
            label,
            &format!(r#"{{ "compilerOptions": {compiler_options}, "files": ["a.ts"] }}"#),
            &[("a.ts", "export const a = 1;\n")],
        )
        .into_iter()
        .filter(|name| name.starts_with("lib."))
        .collect()
    }

    /// Program files that are not lib files.
    fn user_file_names(label: &str, tsconfig: &str, files: &[(&str, &str)]) -> Vec<String> {
        program_file_names(label, tsconfig, files)
            .into_iter()
            .filter(|name| !name.starts_with("lib."))
            .collect()
    }

    // Go `compilerOptions.Lib == nil` loads the default lib. An explicit
    // empty list is not nil, so it loads no lib.
    #[test]
    fn empty_lib_list_loads_no_lib() {
        assert_eq!(
            lib_file_names("empty", r#"{ "lib": [] }"#),
            Vec::<String>::new()
        );
    }

    #[test]
    fn absent_lib_loads_default_lib() {
        let names = lib_file_names("absent", r#"{ "target": "es2015" }"#);
        assert!(names.iter().any(|name| name == "lib.es6.d.ts"), "{names:?}");
    }

    // Go fileloader.go:839 adds the implicit jsx-runtime import only to a
    // JS or JSX file (IsSourceFileJS reads the script kind) or a TSX file.
    // A JSON file also has the JAVA_SCRIPT_FILE flag, but gets no import.
    #[test]
    fn json_file_gets_no_jsx_runtime_import() {
        let names = user_file_names(
            "json_jsx",
            r#"{ "compilerOptions": { "jsx": "react-jsx", "jsxImportSource": "preact",
                 "module": "esnext", "moduleResolution": "bundler",
                 "resolveJsonModule": true, "types": [] },
                 "files": ["a.ts"] }"#,
            &[
                (
                    "a.ts",
                    "import d from \"./d.json\";\nexport const x = d.a;\n",
                ),
                ("d.json", "{ \"a\": 1 }\n"),
                (
                    "node_modules/preact/package.json",
                    "{ \"name\": \"preact\" }\n",
                ),
                ("node_modules/preact/jsx-runtime/index.d.ts", "export {};\n"),
            ],
        );
        assert_eq!(names, ["d.json", "a.ts"]);
    }

    // Go fileloader.go:852 adds the importHelpers tslib import to a JS file
    // or a non-declaration module, never to a JSON file.
    #[test]
    fn json_file_gets_no_tslib_import() {
        let names = user_file_names(
            "json_tslib",
            r#"{ "compilerOptions": { "importHelpers": true, "module": "esnext",
                 "moduleResolution": "bundler", "resolveJsonModule": true, "types": [] },
                 "files": ["a.ts", "d.json"] }"#,
            &[
                ("a.ts", "const x = 1;\n"),
                ("d.json", "{ \"a\": 1 }\n"),
                (
                    "node_modules/tslib/package.json",
                    "{ \"name\": \"tslib\", \"types\": \"tslib.d.ts\" }\n",
                ),
                ("node_modules/tslib/tslib.d.ts", "export {};\n"),
            ],
        );
        assert_eq!(names, ["a.ts", "d.json"]);
    }

    /// Writes `files` and a tsconfig.json with `tsconfig` to a temp dir and
    /// loads the program with parse workers (not single threaded). Returns
    /// the load, the dir and its name; the caller removes the dir.
    fn load_with_workers(
        label: &str,
        tsconfig: &str,
        files: &[(&str, &str)],
    ) -> (ProcessedFiles, std::path::PathBuf, String) {
        load_with_workers_on(label, tsconfig, files, bundled::wrap_fs(osvfs_fs()))
    }

    /// `load_with_workers` with `fs` as the loader's file system: the OS
    /// file system, maybe wrapped (`wrapvfs`). The parse workers read the
    /// OS file system (`run_prefetch_worker`).
    fn load_with_workers_on(
        label: &str,
        tsconfig: &str,
        files: &[(&str, &str)],
        fs: Rc<dyn Fs>,
    ) -> (ProcessedFiles, std::path::PathBuf, String) {
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_file_loader_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::write(dir.join("tsconfig.json"), tsconfig).unwrap();
        let cwd = dir.to_string_lossy().replace('\\', "/");
        let sys = System {
            fs: bundled::wrap_fs(osvfs_fs()),
            current_directory: cwd.clone(),
        };
        let (config, errors) = get_parsed_command_line_of_config_file(
            &format!("{cwd}/tsconfig.json"),
            None,
            None,
            &sys,
            None,
        );
        assert!(errors.is_empty());
        // The parse workers run when the host shows the plain OS file
        // system (`CompilerHost::is_plain_os_fs`): `fs` reads it.
        let host = super::super::host::new_compiler_host_over(
            &cwd,
            crate::frontend::vfs::cachedvfs_from(fs),
            &bundled::wrap_fs(osvfs_fs()),
            &bundled::lib_path(),
        );
        let processed = process_all_program_files(
            ProgramOptions {
                host,
                config: Rc::new(config.unwrap()),
                use_source_of_project_reference: false,
                single_threaded: Tristate::False,
                typings_location: String::new(),
                project_name: String::new(),
                create_module_resolver: None,
                skip_module_resolution: false,
            },
            false,
        );
        (processed, dir, cwd)
    }

    // The parse workers find the package scope of each file for its
    // metadata, and the loader keeps the package.json reads of that walk
    // in the program resolver's cache when it takes the metadata
    // (`Caches::adopt_worker_package_jsons`), as the Go loader finds the
    // metadata with the program's resolver (fileloader.go:391,
    // module/resolver.go:1755 getPackageJsonInfo, packagejson/cache.go:190
    // Set keeps the first). A lookup on the loading thread after the load
    // (Go `Program.GetPackageJsonInfo`, compiler/program.go:157) finds what
    // the load read, also when the files changed on disk since: the
    // project's package.json (the scope of `src/a.ts`) and the package
    // that `src/a.ts` imports (the scope of its `index.d.ts`). With the prep
    // on, the loader waits for the worker of each file (`set_meta_wait`), so
    // it takes the worker's metadata whatever the timing.
    #[test]
    fn the_loader_keeps_the_package_json_files_that_workers_read() {
        let tsconfig = r#"{ "compilerOptions": { "module": "nodenext", "types": [],
             "noEmit": true }, "include": ["src"] }"#;
        let files = [
            ("package.json", r#"{ "name": "app", "type": "module" }"#),
            (
                "src/a.ts",
                "import { x } from \"lib\";\nexport const a = x;\n",
            ),
            (
                "node_modules/lib/package.json",
                r#"{ "name": "lib", "version": "1.0.0", "types": "index.d.ts" }"#,
            ),
            (
                "node_modules/lib/index.d.ts",
                "export declare const x: number;\n",
            ),
        ];
        // With no parse workers (one CPU, or `GOPORT_PARSE_THREADS=0`) the
        // loader reads every file itself.
        let workers = super::super::files_parser::parse_workers_enabled();
        for prep in [true, false] {
            set_load_prep(Some(prep));
            set_meta_wait(prep && workers);
            let before = crate::frontend::module::cache::adopted_package_jsons();
            let (processed, dir, cwd) =
                load_with_workers(&format!("keeps_package_jsons_{prep}"), tsconfig, &files);
            for path in ["package.json", "node_modules/lib/package.json"] {
                std::fs::write(dir.join(path), r#"{ "name": "changed" }"#).unwrap();
            }
            let resolver = processed
                .resolver
                .as_ref()
                .and_then(|resolver| resolver.as_default_resolver())
                .expect("the default resolver");
            // The name in the package.json of the package scope of `path`.
            let name = |path: &str| {
                resolver
                    .get_package_scope_for_path(path)
                    .and_then(|entry| entry.contents.clone())
                    .map(|contents| contents.fields.header_fields.name.get_value().0)
            };
            // The resolver gives a node_modules file by its real path (Go
            // `resolutionState.realPath`, module/resolver.go:1859), so the
            // scope of `lib/index.d.ts` is under the real path of the dir.
            // On macOS the temp dir is under /var, a symlink to /private/var.
            let real = osvfs_fs().realpath(&cwd);
            let names = (
                name(&format!("{cwd}/src")),
                name(&format!("{real}/node_modules/lib")),
            );
            let adopted = crate::frontend::module::cache::adopted_package_jsons() - before;
            let _ = std::fs::remove_dir_all(&dir);
            assert_eq!(
                names,
                (Some("app".to_string()), Some("lib".to_string())),
                "prep {prep}"
            );
            if prep && workers {
                assert!(adopted > 0, "no worker package.json taken");
            }
        }
        set_load_prep(None);
        set_meta_wait(false);
    }

    // followups32: when the loader takes a file's metadata from its parse
    // worker, it keeps the package.json reads of the worker's package scope
    // walk in the program resolver's cache (`Caches::adopt_worker_package_jsons`),
    // so its own lookups later in the load find them and do not read the
    // files again, as the Go loader finds the metadata with the program's
    // resolver (fileloader.go:391, module/resolver.go:1755
    // getPackageJsonInfo, packagejson/cache.go:190 Set). No worker resolves
    // the module augmentation "aug" (the workers resolve imports), so the
    // loader resolves it while it loads `src/a.ts`, after it took that
    // file's metadata, and its self-name lookup (module/resolver.go:576
    // loadModuleFromSelfNameReference) asks for the package scope of `src`,
    // which the worker's walk read. The loader waits for the worker's
    // metadata (`set_meta_wait`), so this holds whatever the timing. Without
    // the take-time adoption the loader reads the project's package.json
    // itself, and the end of the load does not change that.
    #[test]
    fn the_loader_reads_no_package_json_that_a_taken_metadata_walk_read() {
        if !super::super::files_parser::parse_workers_enabled() {
            return;
        }
        let tsconfig = r#"{ "compilerOptions": { "module": "nodenext", "types": [],
             "noEmit": true }, "include": ["src"] }"#;
        let files = [
            ("package.json", r#"{ "name": "app", "type": "module" }"#),
            (
                "src/a.ts",
                "export const a = 1;\ndeclare module \"aug\" {}\n",
            ),
        ];
        // The package.json files that the loader reads.
        let reads = Rc::new(RefCell::new(Vec::<String>::new()));
        let os = bundled::wrap_fs(osvfs_fs());
        let read_file = {
            let (os, reads) = (os.clone(), reads.clone());
            move |path: &str| {
                if path.ends_with("/package.json") {
                    reads.borrow_mut().push(path.to_string());
                }
                os.read_file(path)
            }
        };
        let fs = crate::frontend::vfs::wrapvfs_wrap(
            os,
            crate::frontend::vfs::Replacements {
                read_file: Some(Box::new(read_file)),
                ..Default::default()
            },
        );
        set_load_prep(Some(true));
        set_meta_wait(true);
        let before = crate::frontend::module::cache::adopted_package_jsons();
        let (processed, dir, cwd) =
            load_with_workers_on("no_package_json_reads", tsconfig, &files, fs);
        let adopted = crate::frontend::module::cache::adopted_package_jsons() - before;
        set_load_prep(None);
        set_meta_wait(false);
        let _ = std::fs::remove_dir_all(&dir);
        let resolver = processed
            .resolver
            .as_ref()
            .and_then(|resolver| resolver.as_default_resolver())
            .expect("the default resolver");
        let cache = &resolver.caches.package_json_info_cache;
        assert!(adopted > 0, "no worker package.json taken");
        // A lookup of the load asked for the entry (a read that no lookup
        // asks for stays a pending read, which `contains_key` does not see).
        assert!(
            cache.contains_key(&cache.key(&format!("{cwd}/package.json"))),
            "the loader did not look up the package scope of src"
        );
        assert_eq!(*reads.borrow(), Vec::<String>::new());
    }

    // A parse worker that panics after it starts a job ends the job
    // (`RunningJob`), so the loader that waits for it finds the metadata and
    // parses the file itself, and the load ends with the file. The loader
    // waits for the worker of each file (`set_meta_wait`), so without the
    // guard it would wait forever: the test fails after 60 s. Go has no
    // parse worker; its loader finds the metadata (fileloader.go:391).
    #[test]
    fn a_worker_panic_leaves_no_loader_waiting() {
        use super::super::files_parser::{PANIC_IN_JOB, parse_workers_enabled};
        if !parse_workers_enabled() {
            return;
        }
        let label = "worker_panic";
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_file_loader_{label}_{}",
            std::process::id()
        ));
        let a = format!("{}/src/a.ts", dir.to_string_lossy().replace('\\', "/"));
        *PANIC_IN_JOB.lock().unwrap() = Some(a.clone());
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            set_load_prep(Some(true));
            set_meta_wait(true);
            let tsconfig = r#"{ "compilerOptions": { "module": "nodenext", "types": [],
                 "noEmit": true }, "include": ["src"] }"#;
            let files = [
                ("package.json", r#"{ "name": "app", "type": "module" }"#),
                ("src/a.ts", "export const a = 1;\n"),
            ];
            let (processed, dir, _) = load_with_workers(label, tsconfig, &files);
            let _ = std::fs::remove_dir_all(&dir);
            let a = processed
                .files
                .iter()
                .find(|file| file.file_name().ends_with("/src/a.ts"))
                .map(|file| processed.source_file_meta_datas[file.path()].implied_node_format);
            let _ = sender.send(a);
        });
        let loaded = receiver.recv_timeout(std::time::Duration::from_secs(60));
        *PANIC_IN_JOB.lock().unwrap() = None;
        let loaded =
            loaded.expect("the loader still waits for the job of the worker that panicked");
        assert_eq!(loaded, Some(ModuleKind::ES_NEXT), "{a}");
    }

    // loadcrit2: the job of an output `.d.ts` file of a project reference,
    // which a worker parses in place of its source (`PrefetchQueue::redirects`)
    // and resolves with the reference's redirect. When its worker panics
    // after it starts the job, the loader that waits for it still loads the
    // file and resolves its names with the redirect. Without the wake the
    // test fails after 60 s. The panic ends its worker, and the loader also
    // waits for the jobs that no worker has started (`set_meta_wait`), so
    // the test needs a second worker to run them: with one
    // (`GOPORT_PARSE_THREADS=1`, two CPUs) it does not run.
    #[test]
    fn a_worker_panic_in_a_redirected_job_leaves_no_loader_waiting() {
        use super::super::files_parser::{
            PANIC_IN_JOB, parse_workers_enabled, prefetch_worker_count,
        };
        if !parse_workers_enabled() || prefetch_worker_count() < 2 {
            return;
        }
        let label = "worker_panic_redirect";
        let dir = std::env::temp_dir().join(format!(
            "ts_goport_file_loader_{label}_{}",
            std::process::id()
        ));
        let output = format!(
            "{}/lib/out/helper.d.ts",
            dir.to_string_lossy().replace('\\', "/")
        );
        *PANIC_IN_JOB.lock().unwrap() = Some(output.clone());
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            set_load_prep(Some(true));
            set_meta_wait(true);
            let (processed, dir, _) =
                load_with_workers(label, REFERENCE_TSCONFIG, &REFERENCE_FILES);
            let _ = std::fs::remove_dir_all(&dir);
            let dep = processed
                .files
                .iter()
                .find(|file| file.file_name().ends_with("/lib/out/helper.d.ts"))
                .and_then(|file| processed.resolved_modules.get(file.path()))
                .and_then(|resolutions| {
                    resolutions
                        .iter()
                        .find(|(key, _)| key.name == "dep")
                        .map(|(_, resolved)| get_base_file_name(&resolved.resolved_file_name))
                });
            let _ = sender.send(dep);
        });
        let loaded = receiver.recv_timeout(std::time::Duration::from_secs(60));
        *PANIC_IN_JOB.lock().unwrap() = None;
        let loaded =
            loaded.expect("the loader still waits for the job of the worker that panicked");
        assert_eq!(loaded, Some("lib.d.ts".to_string()), "{output}");
    }

    // lazypj1: the parse workers also read package.json files for the
    // module answers that the loader takes. At the end of the load those
    // reads go into the program resolver's cache, which parses one only
    // when a lookup asks for it (`SharedResolutionCache::end_package_json_reads`,
    // `InfoCache::get`), as Go's parse tasks put every read into the
    // resolver's one cache (module/resolver.go:1755 getPackageJsonInfo,
    // packagejson/cache.go:190 Set keeps the first). The package.json of
    // `lib` is in no file's package scope (its types are under `sub`, which
    // has its own package.json), so only the answer for "lib" reads it. A
    // lookup on the loading thread after the load finds what the load
    // read, also when the file changed on disk since. With the prep on, the
    // loader waits for the worker's metadata (`set_meta_wait`) and answer
    // (`set_answer_wait`), so it takes the answer for "lib" and does not
    // read the file, whatever the timing.
    #[test]
    fn the_loader_keeps_the_package_json_files_of_worker_module_answers() {
        let tsconfig = r#"{ "compilerOptions": { "module": "nodenext", "types": [],
             "noEmit": true }, "include": ["src"] }"#;
        let files = [
            ("package.json", r#"{ "name": "app", "type": "module" }"#),
            (
                "src/a.ts",
                "import { x } from \"lib\";\nexport const a = x;\n",
            ),
            (
                "node_modules/lib/package.json",
                r#"{ "name": "lib", "version": "1.0.0", "types": "sub/index.d.ts" }"#,
            ),
            ("node_modules/lib/sub/package.json", r#"{ "name": "sub" }"#),
            (
                "node_modules/lib/sub/index.d.ts",
                "export declare const x: number;\n",
            ),
        ];
        // With no parse workers the loader reads every file itself.
        let workers = super::super::files_parser::parse_workers_enabled();
        for prep in [true, false] {
            set_load_prep(Some(prep));
            set_meta_wait(prep && workers);
            set_answer_wait(prep && workers);
            let (processed, dir, cwd) = load_with_workers(
                &format!("keeps_answer_package_jsons_{prep}"),
                tsconfig,
                &files,
            );
            std::fs::write(
                dir.join("node_modules/lib/package.json"),
                r#"{ "name": "changed" }"#,
            )
            .unwrap();
            let resolver = processed
                .resolver
                .as_ref()
                .and_then(|resolver| resolver.as_default_resolver())
                .expect("the default resolver");
            let cache = &resolver.caches.package_json_info_cache;
            let package_json = format!("{cwd}/node_modules/lib/package.json");
            // The loader took the answer for "lib" and did not read the file.
            let loader_read = cache.contains_key(&cache.key(&package_json));
            let name = resolver
                .get_package_scope_for_path(&format!("{cwd}/node_modules/lib"))
                .and_then(|entry| entry.contents.clone())
                .map(|contents| contents.fields.header_fields.name.get_value().0);
            let _ = std::fs::remove_dir_all(&dir);
            assert_eq!(name, Some("lib".to_string()), "prep {prep}");
            if prep && workers {
                assert!(!loader_read, "the loader read the package.json of lib");
            }
        }
        set_load_prep(None);
        set_meta_wait(false);
        set_answer_wait(false);
    }

    /// A fake content mapper process over a socket pair (Go `net.Pipe` in
    /// the Go host tests). It answers `initialize` and `openProject` at
    /// once, and the transform requests newest first on another thread, so
    /// the answers come out of order when requests overlap. The text of a
    /// mapped file is TypeScript: `@diag` adds a mapper diagnostic, `@fail`
    /// answers with an error, `@badmap` gives a mapping that fails
    /// `SpanMap::validate`, `@supp` adds a supplemental output. After
    /// `exit_after` transform requests it closes the connection (a mapper
    /// crash). `transforms` gets the file name of each transform request.
    #[cfg(unix)]
    mod fake_mapper {
        use crate::contentmapper::{
            Diagnostic, InitializeResult, MappedOutput, OpenProjectResult, PositionEncoding,
            ProcessExitState, SupplementalOutput, TransformParams, TransformResult,
        };
        use crate::frontend::json::json_unmarshal;
        use crate::frontend::json_ext::{AnyValue, JsonValue};
        use crate::gostd::GoError;
        use crate::ipc::{self, Message, Protocol as _, ReadWriteCloser};
        use std::io::{Read, Write};
        use std::sync::{Arc, Condvar, Mutex};

        #[cfg(unix)]
        type Stream = std::os::unix::net::UnixStream;
        // Windows has no socket pair in std: a connected loopback TCP pair
        // gives the same two-ended byte stream.
        #[cfg(windows)]
        type Stream = std::net::TcpStream;

        #[cfg(unix)]
        fn stream_pair() -> std::io::Result<(Stream, Stream)> {
            Stream::pair()
        }

        #[cfg(windows)]
        fn stream_pair() -> std::io::Result<(Stream, Stream)> {
            let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
            let client = Stream::connect(listener.local_addr()?)?;
            let (server, _) = listener.accept()?;
            Ok((client, server))
        }

        struct End(Stream);

        impl ReadWriteCloser for End {
            fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
                (&self.0).read(buf)
            }

            fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
                (&self.0).write(buf)
            }

            fn flush(&self) -> std::io::Result<()> {
                (&self.0).flush()
            }

            fn close(&self) -> Result<(), GoError> {
                let _ = self.0.shutdown(std::net::Shutdown::Both);
                Ok(())
            }
        }

        impl ProcessExitState for End {}

        type Queue = Arc<(Mutex<(Vec<Message>, bool)>, Condvar)>;

        pub(super) fn spawn(
            transforms: Arc<Mutex<Vec<String>>>,
            exit_after: Option<usize>,
        ) -> Arc<dyn ProcessExitState> {
            let (client, server) = stream_pair().expect("socket pair");
            let server: Arc<dyn ReadWriteCloser> = Arc::new(End(server));
            let write = Arc::new(Mutex::new(()));
            let queue: Queue = Arc::default();
            let answer = {
                let (server, write) = (server.clone(), write.clone());
                move |msg: &Message, result: Option<Box<dyn AnyValue>>| {
                    let _write = write.lock().unwrap();
                    let _ = ipc::new_jsonrpc_protocol(server.clone())
                        .write_response(msg.id.as_ref(), result);
                }
            };
            {
                let (server, write, queue) = (server.clone(), write, queue.clone());
                std::thread::spawn(move || {
                    loop {
                        let msg = {
                            let (lock, ready) = &*queue;
                            let mut queue = lock.lock().unwrap();
                            loop {
                                if let Some(msg) = queue.0.pop() {
                                    break msg;
                                }
                                if queue.1 {
                                    return;
                                }
                                queue = ready.wait(queue).unwrap();
                            }
                        };
                        let mut params = TransformParams::default();
                        json_unmarshal(&msg.params.0, &mut params, &[]).unwrap();
                        let _write = write.lock().unwrap();
                        let mut protocol = ipc::new_jsonrpc_protocol(server.clone());
                        if params.content.contains("@fail") {
                            let _ = protocol.write_error(
                                msg.id.as_ref(),
                                &crate::jsonrpc::ResponseError {
                                    code: crate::jsonrpc::CODE_INTERNAL_ERROR,
                                    message: "fake failure".to_string(),
                                    data: None,
                                },
                            );
                            continue;
                        }
                        let _ = protocol.write_response(msg.id.as_ref(), Some(transform(&params)));
                    }
                });
            }
            std::thread::spawn(move || {
                let mut protocol = ipc::new_jsonrpc_protocol(server.clone());
                let mut count = 0;
                while let Ok(msg) = protocol.read_message() {
                    match msg.method.as_str() {
                        "initialize" => answer(
                            &msg,
                            Some(Box::new(InitializeResult {
                                position_encoding: PositionEncoding::UTF8,
                                diagnostic_source: "fake".to_string(),
                            })),
                        ),
                        "openProject" => {
                            answer(&msg, Some(Box::new(OpenProjectResult::default())));
                        }
                        "closeProject" => answer(&msg, None),
                        "transform" => {
                            count += 1;
                            if exit_after.is_some_and(|limit| count > limit) {
                                let _ = server.close();
                                break;
                            }
                            let mut params = TransformParams::default();
                            json_unmarshal(&msg.params.0, &mut params, &[]).unwrap();
                            transforms.lock().unwrap().push(params.file_name);
                            let (lock, ready) = &*queue;
                            lock.lock().unwrap().0.push(msg);
                            ready.notify_one();
                        }
                        _ => {}
                    }
                }
                let (lock, ready) = &*queue;
                lock.lock().unwrap().1 = true;
                ready.notify_one();
            });
            Arc::new(End(client))
        }

        /// `text` mapped to itself, up to `end`.
        fn verbatim(text: &str, extension: &str, end: usize) -> MappedOutput {
            let mappings = crate::spanmap::new(&[crate::spanmap::Segment {
                virtual_end: end as i32,
                original_end: text.len() as i32,
                kind: crate::spanmap::Kind::VERBATIM,
                ..Default::default()
            }])
            .marshal()
            .unwrap();
            MappedOutput {
                text: text.to_string(),
                extension: extension.to_string(),
                mappings: JsonValue(mappings),
                diagnostic_directives: None,
            }
        }

        fn transform(params: &TransformParams) -> Box<dyn AnyValue> {
            // `@badmap`: the verbatim mapping ends one byte before the
            // virtual text, so its two lengths differ. The decode of the
            // answer takes it, and `SpanMap::validate` fails it.
            let end = params.content.len() - usize::from(params.content.contains("@badmap"));
            let mut result = TransformResult {
                mapped_output: verbatim(&params.content, ".ts", end),
                ..Default::default()
            };
            if params.content.contains("@diag") {
                result.diagnostics.push(Diagnostic {
                    message_text: "fake diagnostic".to_string(),
                    start: 0,
                    length: 1,
                    code: 9001,
                });
            }
            if params.content.contains("@supp") {
                result.supplemental.push(SupplementalOutput {
                    mapped_output: MappedOutput {
                        text: "export const supplemental = 1;\n".to_string(),
                        extension: ".ts".to_string(),
                        mappings: JsonValue(b"[]".to_vec()),
                        diagnostic_directives: None,
                    },
                });
            }
            Box::new(result)
        }
    }

    /// Loads a program of `files` in a temp dir with the fake mapper for
    /// `.vue` files. Returns each program file (name, text and parse
    /// diagnostics) and the content mapper diagnostics, one per line, the
    /// file name of each transform request, the transforms that the loader
    /// took from the parse workers (file name, and whether a worker parse
    /// came with it), and the spawns.
    #[cfg(unix)]
    fn load_mapped(
        label: &str,
        tsconfig: &str,
        files: &[(String, String)],
        single_threaded: bool,
        exit_after: Option<usize>,
    ) -> (Vec<String>, Vec<String>, Vec<(String, bool)>, usize) {
        use crate::contentmapper::{self, HostOptions, ProjectSpec, SpawnerFunc};
        let dir = std::path::PathBuf::from(mapped_dir(label));
        let _ = std::fs::remove_dir_all(&dir);
        let mapper_package = (
            "node_modules/fake-mapper/package.json".to_string(),
            r#"{ "name": "fake-mapper", "version": "1.0.0",
                 "typescript": { "contentMapper": { "exec": ["fake"] } } }"#
                .to_string(),
        );
        for (path, text) in files.iter().chain([&mapper_package]) {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::write(dir.join("tsconfig.json"), tsconfig).unwrap();
        let cwd = dir.to_string_lossy().replace('\\', "/");
        let fs = bundled::wrap_fs(osvfs_fs());
        let sys = System {
            fs: fs.clone(),
            current_directory: cwd.clone(),
        };
        // Go `tsc --runExternalCode`.
        let options = CompilerOptions {
            run_external_code: Tristate::True,
            ..Default::default()
        };
        let (config, errors) = get_parsed_command_line_of_config_file(
            &format!("{cwd}/tsconfig.json"),
            Some(&options),
            None,
            &sys,
            None,
        );
        assert!(errors.is_empty());
        let config = Rc::new(config.unwrap());
        assert_eq!(config.content_mappers().len(), 1);
        let transforms = Arc::new(std::sync::Mutex::new(Vec::new()));
        let spawns = Rc::new(Cell::new(0));
        let spawner = {
            let (transforms, spawns) = (transforms.clone(), spawns.clone());
            SpawnerFunc(Box::new(move |_, _, _| {
                spawns.set(spawns.get() + 1);
                Ok(fake_mapper::spawn(transforms.clone(), exit_after))
            }))
        };
        let ctx = crate::gostd::context::background();
        let host = contentmapper::new_host_with_options(
            &ctx,
            Rc::new(spawner),
            crate::locale::DEFAULT,
            HostOptions { logger: None },
        );
        let project = host.project(ProjectSpec {
            config_file_name: config.config_name().to_string(),
            mappers: config.content_mappers().to_vec(),
            compiler_options: Some(config.compiler_options().clone()),
        });
        let compiler_host = new_cached_fs_compiler_host(
            &cwd,
            fs,
            &bundled::lib_path(),
            None,
            None,
            project.clone(),
        );
        let processed = process_all_program_files(
            ProgramOptions {
                host: compiler_host,
                config,
                use_source_of_project_reference: false,
                single_threaded: if single_threaded {
                    Tristate::True
                } else {
                    Tristate::False
                },
                typings_location: String::new(),
                project_name: String::new(),
                create_module_resolver: None,
                skip_module_resolution: false,
            },
            single_threaded,
        );
        if let Some(project) = project {
            let _ = project.close();
        }
        let _ = host.close();
        let _ = std::fs::remove_dir_all(&dir);
        let diagnostic = |d: &Diagnostic| {
            format!(
                "{:?}",
                (
                    d.pos,
                    d.end,
                    d.code,
                    &d.source,
                    &d.message_text,
                    &d.message_args
                )
            )
        };
        let mut lines: Vec<String> = processed
            .files
            .iter()
            .filter(|file| file.file_name().starts_with(&cwd))
            .map(|file| {
                let name = file.file_name().replace(&cwd, "");
                let diagnostics: Vec<String> = file.diagnostics.iter().map(diagnostic).collect();
                format!("{name} {:?} {diagnostics:?}", file.text())
            })
            .collect();
        lines.extend(processed.content_mapper_diagnostics.iter().map(diagnostic));
        let transforms = std::mem::take(&mut *transforms.lock().unwrap());
        let transforms = transforms
            .into_iter()
            .map(|name| name.replace(&cwd, ""))
            .collect();
        let taken = super::super::files_parser::MAPPED_TAKEN
            .lock()
            .unwrap()
            .extract_if(.., |(name, _)| name.starts_with(&cwd))
            .map(|(name, parsed)| (name.replace(&cwd, ""), parsed))
            .collect();
        (lines, transforms, taken, spawns.get())
    }

    /// The temp dir of the `load_mapped` run `label`, as the loader names it.
    #[cfg(unix)]
    fn mapped_dir(label: &str) -> String {
        std::env::temp_dir()
            .join(format!(
                "ts_goport_file_loader_{label}_{}",
                std::process::id()
            ))
            .to_string_lossy()
            .replace('\\', "/")
    }

    #[cfg(unix)]
    const MAPPED_TSCONFIG: &str = r#"{ "compilerOptions": { "module": "preserve",
         "moduleResolution": "bundler", "types": [], "noEmit": true },
         "include": ["src"],
         "contentMappers": [{ "package": "fake-mapper", "extensions": [".vue"] }] }"#;

    /// `count` mapped files that import the next one, with `marker` in
    /// file `i` when `marker(i)` names one.
    #[cfg(unix)]
    fn mapped_files(count: usize, marker: fn(usize) -> &'static str) -> Vec<(String, String)> {
        let mut files: Vec<(String, String)> = (0..count)
            .map(|i| {
                let next = (i + 1) % count;
                (
                    format!("src/C{i}.vue"),
                    format!(
                        "// {}\nimport {{ v{next} }} from \"./C{next}.vue\";\nexport const v{i} = v{next};\n",
                        marker(i)
                    ),
                )
            })
            .collect();
        files.push((
            "src/main.ts".to_string(),
            "import { v0 } from \"./C0.vue\";\nexport const main = v0;\n".to_string(),
        ));
        files
    }

    // cmpar1: once the loader's transform of the first mapped file opened
    // the mapper project, the parse workers send the transforms of the
    // other mapped files and parse the virtual texts, and the loader takes
    // their results. Each file gets one transform request, also with a
    // mapper diagnostic, an error response, a mapping that fails
    // `SpanMap::validate` (the worker does not parse it, and the loader
    // sends no second request for it) or a supplemental output, and
    // the program equals a load on one thread. The fake mapper answers the
    // newest request first, so answers come out of order when requests
    // overlap. Which files the workers take varies, so the loader leaves
    // the jobs of the `@badmap` file and of 3 plain files to the workers
    // (`MAPPED_WAIT_FOR_WORKER`): a worker sends their transforms and parses
    // the plain files, also on a loaded host where the loader would reach
    // them first. The parallel load still runs again, at most 10 times,
    // until a worker sent the transform of the `@badmap` file. Go sends the
    // transforms from its parse goroutines (fileloader.go:438
    // parseContentMappedFile), and parses only after the mapping check
    // (transform.go:48-58 ParseResult).
    #[cfg(unix)]
    #[test]
    fn workers_send_the_content_mapper_transforms() {
        use super::super::files_parser::{MAPPED_WAIT_FOR_WORKER, parse_workers_enabled};
        let files = mapped_files(40, |i| match i {
            3 | 17 | 31 => "@diag",
            9 | 26 => "@fail",
            21 => "@badmap",
            12 | 35 => "@supp",
            _ => "",
        });
        let (serial, mut serial_transforms, _, _) =
            load_mapped("mapped_serial", MAPPED_TSCONFIG, &files, true, None);
        // One request per mapped file: a file with two requests, or with
        // none, fails here.
        let mut want: Vec<String> = (0..40).map(|i| format!("/src/C{i}.vue")).collect();
        want.sort();
        serial_transforms.sort();
        assert_eq!(serial_transforms, want);
        for attempt in 0..10 {
            let label = format!("mapped_parallel_{attempt}");
            let held: Vec<String> = [21, 5, 15, 30]
                .iter()
                .map(|i| format!("{}/src/C{i}.vue", mapped_dir(&label)))
                .collect();
            if parse_workers_enabled() {
                MAPPED_WAIT_FOR_WORKER.lock().unwrap().extend(held.clone());
            }
            let (parallel, mut parallel_transforms, taken, _) =
                load_mapped(&label, MAPPED_TSCONFIG, &files, false, None);
            MAPPED_WAIT_FOR_WORKER
                .lock()
                .unwrap()
                .retain(|name| !held.contains(name));
            parallel_transforms.sort();
            assert_eq!(parallel_transforms, want, "one transform per file");
            assert_eq!(parallel.join("\n"), serial.join("\n"));
            if !parse_workers_enabled() {
                return;
            }
            assert!(
                taken.iter().any(|(_, parsed)| *parsed),
                "the loader took no worker parse: {taken:?}"
            );
            if let Some((_, parsed)) = taken.iter().find(|(name, _)| name == "/src/C21.vue") {
                assert!(
                    !parsed,
                    "a worker parsed the virtual text of a failed mapping"
                );
                return;
            }
        }
        panic!("no worker sent the transform of the @badmap file in 10 loads");
    }

    // A mapper that crashes during the load: the calls in flight and all
    // later calls fail, the first 5 failures in load order are reported,
    // then the mapper is disabled, and the load ends.
    #[cfg(unix)]
    #[test]
    fn a_mapper_crash_during_worker_transforms_ends_the_load() {
        let files = mapped_files(40, |_| "");
        for single_threaded in [true, false] {
            let (lines, _, _, _) = load_mapped(
                &format!("mapped_crash_{single_threaded}"),
                MAPPED_TSCONFIG,
                &files,
                single_threaded,
                Some(10),
            );
            let count = |code: u32| {
                lines
                    .iter()
                    .filter(|line| line.contains(&format!(", {code}, ")))
                    .count()
            };
            let failed = diag::The_content_mapper_0_failed_to_transform_this_file.code();
            let disabled = diag::The_content_mapper_0_failed_1_times_and_will_not_be_used.code();
            assert_eq!(
                (count(failed), count(disabled)),
                (5, 1),
                "single threaded {single_threaded}:\n{}",
                lines.join("\n")
            );
        }
    }

    // A mapped file that the program names but that does not exist sends
    // no request: the mapper does not start, so the project does not open
    // and reports no option diagnostics (Go program.go:828 reports them
    // only for opened projects).
    #[cfg(unix)]
    #[test]
    fn a_missing_mapped_file_starts_no_mapper() {
        let tsconfig = r#"{ "compilerOptions": { "types": [], "noEmit": true },
             "files": ["missing.vue", "a.ts"],
             "contentMappers": [{ "package": "fake-mapper", "extensions": [".vue"] }] }"#;
        let files = [("a.ts".to_string(), "export const a = 1;\n".to_string())];
        for single_threaded in [true, false] {
            let (_, transforms, _, spawns) = load_mapped(
                &format!("mapped_missing_{single_threaded}"),
                tsconfig,
                &files,
                single_threaded,
                None,
            );
            assert_eq!((transforms.len(), spawns), (0, 0));
        }
    }
}
