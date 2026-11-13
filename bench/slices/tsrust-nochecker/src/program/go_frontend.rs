//! The goport program loader: Go tsc config parsing
//! (`tsoptions.GetParsedCommandLineOfConfigFile`), the Go program loader
//! (`compiler.NewProgram`: scanner, parser, module resolution, file order)
//! and the `GoProgram` built from its node stores.
//!
//! Go: execute/tsc.go:213 (config), tsc.go:293 (host), tsc.go:301 (program).

use super::*;
use crate::ast::store::{
    
};
use crate::frontend::bundled;
use crate::frontend::compiler::{
    
    
};
use crate::frontend::parser::{SourceFileParseOptions};
use crate::frontend::tsoptions::{
    
};
use crate::frontend::tspath;
use crate::frontend::tspath::Path as GoPath;
use crate::frontend::vfs::{Fs, };
use crate::rustc_hash::FxHashSet;
use std::rc::Rc;










/// Go `tsc.System` as `ParseConfigHost` (FS and current directory).
struct System {
    fs: Rc<dyn Fs>,
    current_directory: String,
}






// Go: compiler/program.go:2300 GetSymlinkCache (`get_go_symlink_cache`)
// PORT: Go builds the cache on first use, on any goroutine. The port
// builds it from the frontend program, which only the loading thread has,
// and keeps a thread-safe copy in the program's shared state. The loading
// thread builds it on first use, or when it seeds a thread
// (`build_lazy_shared_state`), so a seeded thread finds it built.








// Go: ast/ast.go:2745 (*SourceFile).resolveJSDoc (slow path; the caller
// has checked the parser cache) and parser/jsdoc.go:19 parseJSDocForNode


// Go: compiler/program.go:129 FileExists (the Go frontend program)
// PORT: the loading thread asks the program host (with its cache). Another
// thread (a checker worker) cannot read that cache. It asks the program's
// thread-safe cache (Go `cachedvfs.FS`, compiler/host.go:52), which asks
// this thread's copy of the file system on a miss. Module specifier
// generation probes the same paths for many types (`tryGetAnyFileFromPath`):
// realworld3 lexical made 8.6M uncached stat calls, Go 8,768.





/// How `build_program` makes a program known.
enum Entry {
    /// The program of a one-program process (`core::set_prog`).
    Only,
    /// A program version of a multi-program process
    /// (`core::register_program_version`). The caller parses and builds it
    /// inside `core::enter_program(None)`.
    Version,
}


/// True in debug builds, or when `GOPORT_CHECK_VERSION_TABLES=1`:
/// `build_program` builds the tables of a version that starts from the
/// tables of an older one (`reused_tables`) again from its files alone
/// (`full_tables`), and panics when they differ. Read once.
fn check_version_tables() -> bool {
    static CHECK: OnceLock<bool> = OnceLock::new();
    *CHECK.get_or_init(|| {
        cfg!(debug_assertions)
            || std::env::var("GOPORT_CHECK_VERSION_TABLES").is_ok_and(|v| v == "1")
    })
}









// Go: compiler/program.go:1562 CommonSourceDirectory.
// PORT: Go computes it on first use, and `checkSourceFilesBelongToPath`
// then adds include diagnostics. Go uses it first either in
// `verifyCompilerOptions`, which the frontend program has already run, or
// during emit, after the program diagnostics are reported. So the value is
// computed when the program is built, without the check, which adds
// nothing.

// Go: compiler/program.go:828 collectContentMapperOptionDiagnostics (#4712)
// PORT: returns the list; Go sets `p.contentMapperOptionDiagnostics`.









#[cfg(test)]
mod tests {
    use super::*;

    /// A copy of `np` with the file at `slot` parsed again, as Go
    /// `ReuseProgram` replaces a changed file in place: the same files at
    /// the same paths and the same resolutions.
    fn replaced_in_place(np: &NewProgram, slot: usize) -> NewProgram {
        let mut processed_files = np.processed_files.clone();
        let old = &processed_files.files[slot];
        let file = Rc::new(crate::frontend::parser::parse_source_file(
            old.parse_options(),
            crate::ast::file_version::FileText::new(format!("{}\n", old.text()), false),
            old.script_kind,
        ));
        processed_files.files[slot] = file.clone();
        processed_files
            .files_by_path
            .insert(file.path().clone(), file);
        NewProgram {
            opts: np.opts.clone(),
            compare_paths_options: np.compare_paths_options.clone(),
            processed_files,
            uses_uri_style_node_core_modules: np.uses_uri_style_node_core_modules,
            common_source_directory: std::cell::OnceCell::new(),
            program_diagnostics: Vec::new(),
            has_emit_blocking_diagnostics: FxHashSet::default(),
            source_files_to_emit: std::cell::OnceCell::new(),
            unresolved_imports: Default::default(),
            known_symlinks: Default::default(),
            package_names: Default::default(),
            has_ts_file: std::cell::OnceCell::new(),
        }
    }

    /// A program whose `index.ts` imports two packages, each with its own
    /// copy of pkg@1.0.0: a package redirect group. The second copy's path
    /// is a redirect path, and the first copy is its target. `name` names
    /// the fixture dir. The caller enters the program scope.
    fn redirect_group_program(name: &str) -> NewProgram {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let pkg = r#"{"name":"pkg","version":"1.0.0","types":"index.d.ts"}"#;
        let files = [
            ("tsconfig.json", r#"{"compilerOptions":{"types":[]}}"#),
            (
                "index.ts",
                "import { a } from \"a\";\nimport { b } from \"b\";\nexport const x = a + b;\n",
            ),
            (
                "node_modules/a/package.json",
                r#"{"name":"a","types":"index.d.ts"}"#,
            ),
            (
                "node_modules/a/index.d.ts",
                "import { p } from \"pkg\";\nexport declare const a: typeof p;\n",
            ),
            ("node_modules/a/node_modules/pkg/package.json", pkg),
            (
                "node_modules/a/node_modules/pkg/index.d.ts",
                "export declare const p: number;\n",
            ),
            (
                "node_modules/b/package.json",
                r#"{"name":"b","types":"index.d.ts"}"#,
            ),
            (
                "node_modules/b/index.d.ts",
                "import { p } from \"pkg\";\nexport declare const b: typeof p;\n",
            ),
            ("node_modules/b/node_modules/pkg/package.json", pkg),
            (
                "node_modules/b/node_modules/pkg/index.d.ts",
                "export declare const p: number;\n",
            ),
        ];
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let config = format!("{}/tsconfig.json", dir.to_string_lossy().replace('\\', "/"));
        let opts = load_config(&config, |_| {}, &mut CompileTimes::default()).unwrap();
        let np = new_program(opts);
        let _ = std::fs::remove_dir_all(&dir);
        np
    }

    // followups4 (R153 reviewer): editfast1 gives a full build when a
    // replaced file is in a package redirect group (`replaced_files`). Go's
    // ReuseProgram checks only the changed file, not the content-mapper
    // supplemental files, so a reused program can still replace a file of
    // a redirect group. Here the same pkg@1.0.0 is in two node_modules
    // dirs, so one copy redirects to the other.
    #[test]
    fn replaced_file_in_redirect_group_takes_the_full_build() {
        let _scope = crate::core::enter_program(None);
        let np = redirect_group_program("goport-redirect");

        let files = np.source_files();
        let old = VersionTables::new((0..files.len()).collect(), std::iter::empty());
        let slot_of = |name: &str| {
            files
                .iter()
                .position(|file| file.file_name().ends_with(name))
                .unwrap_or_else(|| panic!("no {name}"))
        };
        let in_group = |path: &GoPath| {
            np.redirect_files_by_path
                .as_ref()
                .is_some_and(|redirects| redirects.contains_key(path))
                || np
                    .redirect_targets_map
                    .as_ref()
                    .is_some_and(|targets| targets.contains_key(path))
        };
        let pkg = slot_of("/node_modules/a/node_modules/pkg/index.d.ts");
        assert!(
            in_group(files[pkg].path()),
            "pkg is not in a redirect group"
        );
        let index = slot_of("/index.ts");
        assert!(!in_group(files[index].path()));

        // A replaced file outside the group keeps the old tables.
        let edited = replaced_in_place(&np, index);
        let replaced = replaced_files(&edited, &np, &old).expect("a reused program");
        assert_eq!(replaced.iter().map(|r| r.slot).collect::<Vec<_>>(), [index]);

        // A replaced file of the group takes the full build.
        let edited = replaced_in_place(&np, pkg);
        assert!(replaced_files(&edited, &np, &old).is_none());
    }

    /// A project in a new dir under the system temp dir: `index.ts`
    /// imports `pkg`, and `node_modules/pkg` is a symlink to `real/pkg`, so
    /// the resolution goes through the symlink. Returns the dir with `/`
    /// separators. `name` names the dir.
    #[cfg(unix)]
    fn symlinked_package_project(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let files = [
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"types":[]},"files":["index.ts"]}"#,
            ),
            (
                "index.ts",
                "import { p } from \"pkg\";\nexport const x = p;\n",
            ),
            (
                "real/pkg/package.json",
                r#"{"name":"pkg","types":"index.d.ts"}"#,
            ),
            ("real/pkg/index.d.ts", "export declare const p: number;\n"),
        ];
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        std::fs::create_dir_all(dir.join("node_modules")).unwrap();
        std::os::unix::fs::symlink("../real/pkg", dir.join("node_modules/pkg")).unwrap();
        dir.to_string_lossy().replace('\\', "/")
    }

    /// The frontend program of `version` on this thread.
    fn frontend_of(version: &GoProgram) -> Rc<NewProgram> {
        FRONTENDS
            .with(|frontends| frontends.borrow().get(&version.id).cloned())
            .expect("a version loaded on this thread")
    }

    // progtables1: Go builds the symlink cache on first use
    // (program.go:2300 GetSymlinkCache, a `lazyValue`). A load does not
    // build it. A seeded thread cannot build it, so the loading thread
    // builds it when it seeds one, and both read the same copy.
    #[cfg(unix)]
    #[test]
    fn symlink_cache_is_built_on_first_use() {
        let dir = symlinked_package_project("goport-lazy-symlinks");
        let program = crate::program::try_load_version(&format!("{dir}/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let frontend = frontend_of(program);
        assert!(
            frontend.known_symlinks.get().is_none(),
            "the load built the symlink cache"
        );
        let (seeded, built) = {
            let _scope = crate::core::enter_program(Some(program));
            let seeded = crate::program::spawn_seeded_thread(|| {
                crate::program::get_go_symlink_cache().expect("a Go frontend program")
            })
            .join()
            .expect("the seeded thread reads the cache");
            let built = crate::program::get_go_symlink_cache().expect("a Go frontend program");
            (seeded, built)
        };
        assert!(Arc::ptr_eq(&seeded, &built), "two copies of the cache");
        let directories = |symlinks: &crate::modulespecifiers::symlinks::KnownSymlinks| {
            symlinks.directories().keys().cloned().collect::<Vec<_>>()
        };
        assert_eq!(
            directories(&built),
            directories(&frontend.get_symlink_cache())
        );
        assert!(
            directories(&built)
                .iter()
                .any(|path| path.0.ends_with("/node_modules/pkg/")),
            "node_modules/pkg is not a known symlink"
        );
        drop(frontend);
        crate::program::release_program(program);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // progtables1: Go `ReuseProgram` takes the old program's symlink cache
    // when it is built (`tryReuse`, program.go:415), and keeps it unbuilt
    // when it is not.
    #[cfg(unix)]
    #[test]
    fn reused_program_shares_a_built_symlink_cache() {
        let dir = symlinked_package_project("goport-reused-symlinks");
        let index = format!("{dir}/index.ts");
        let v1 = crate::program::try_load_version(&format!("{dir}/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let body_edit = |old, digit: &str| {
            std::fs::write(
                &index,
                format!("import {{ p }} from \"pkg\";\nexport const x = p + {digit};\n"),
            )
            .unwrap();
            let (version, reused) = crate::program::update_program_version(old, &index);
            assert!(reused, "a body edit replaces the file in place");
            version
        };
        let v2 = body_edit(v1, "1");
        assert!(frontend_of(v2).known_symlinks.get().is_none());
        let built = {
            let _scope = crate::core::enter_program(Some(v2));
            crate::program::get_go_symlink_cache().expect("a Go frontend program")
        };
        let v3 = body_edit(v2, "2");
        let shared = {
            let _scope = crate::core::enter_program(Some(v3));
            crate::program::get_go_symlink_cache().expect("a Go frontend program")
        };
        assert!(
            Arc::ptr_eq(&built, &shared),
            "the reused program built its own copy"
        );
        for version in [v1, v2, v3] {
            crate::program::release_program(version);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    // progtables1: Go reads a parse file redirect per call (program.go:211
    // GetParseFileRedirect). The copy answers per call from the project
    // reference maps, as the frontend program does. Here b references a,
    // so a's source file redirects to its output d.ts (tsc does not use
    // the source of a project reference).
    #[test]
    fn parse_file_redirects_match_the_frontend() {
        let dir =
            std::env::temp_dir().join(format!("goport-parse-redirects-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let files = [
            (
                "a/tsconfig.json",
                r#"{"compilerOptions":{"composite":true,"outDir":"out","types":[]}}"#,
            ),
            ("a/index.ts", "export const a = 1;\n"),
            ("a/out/index.d.ts", "export declare const a = 1;\n"),
            (
                "b/tsconfig.json",
                r#"{"compilerOptions":{"types":[]},"references":[{"path":"../a"}],"files":["index.ts"]}"#,
            ),
            (
                "b/index.ts",
                "import { a } from \"../a/index\";\nexport const b = a;\n",
            ),
        ];
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let dir = dir.to_string_lossy().replace('\\', "/");
        let program = crate::program::try_load_version(&format!("{dir}/b/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let frontend = frontend_of(program);
        let mut names: Vec<String> = frontend
            .resolved_modules
            .values()
            .flat_map(|cache| cache.values())
            .map(|resolved| resolved.resolved_file_name.clone())
            .filter(|name| !name.is_empty())
            .collect();
        names.extend([format!("{dir}/a/index.ts"), format!("{dir}/b/index.ts")]);
        let mut redirected = 0;
        {
            let _scope = crate::core::enter_program(Some(program));
            for name in &names {
                let want = frontend.get_parse_file_redirect(name);
                let got = with_go(|go| go.get_parse_file_redirect(name).map(str::to_string));
                assert_eq!(got.unwrap_or_default(), want, "redirect of {name}");
                redirected += usize::from(!want.is_empty());
            }
        }
        assert!(redirected > 0, "no name has a redirect");
        drop(frontend);
        crate::program::release_program(program);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // Why the `redirect_files_by_path` half of the bailout in
    // `replaced_files` cannot hold: no program file has a redirect path,
    // so a replaced program file is never at one.
    #[test]
    fn redirect_paths_are_not_program_files() {
        let _scope = crate::core::enter_program(None);
        let np = redirect_group_program("goport-redirect-paths");
        let redirects = np
            .redirect_files_by_path
            .as_ref()
            .expect("a redirect path for the second copy of pkg");
        assert!(
            redirects.keys().any(|path| path
                .0
                .ends_with("/node_modules/b/node_modules/pkg/index.d.ts")),
            "the second copy of pkg is not a redirect path"
        );
        let files = np.source_files();
        assert!(
            files.iter().any(|file| file
                .path()
                .0
                .ends_with("/node_modules/a/node_modules/pkg/index.d.ts")),
            "the first copy of pkg is not a program file"
        );
        for file in files {
            assert!(
                !redirects.contains_key(file.path()),
                "program file {} has a redirect path",
                file.path().0
            );
        }
        // Each redirect path still finds its target (Go `filesByPath`).
        for (path, redirect) in redirects.iter() {
            let target = np
                .files_by_path()
                .get(path)
                .expect("a file at the redirect path");
            assert_eq!(target.path(), &redirect.target);
        }
    }

    /// Writes `files` (path, text) to a new dir under the system temp dir
    /// and returns the dir with `/` separators. `name` names the dir.
    fn write_project(name: &str, files: &[(&str, &str)]) -> String {
        let dir = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir.to_string_lossy().replace('\\', "/")
    }

    // specstat1 (realworld3 gap 1): a thread without the frontend program
    // (a checker worker) asks the program's `HostFsCache`, as Go checkers
    // ask the program host's `cachedvfs.FS` (compiler/host.go:52). So the
    // first answer for a path stays for the program version. Before, each
    // call asked the disk: lexical made 8.6M stat calls, Go 8,768.
    #[test]
    fn worker_file_exists_keeps_the_first_answer() {
        let dir = write_project(
            "goport-worker-file-exists",
            &[
                (
                    "tsconfig.json",
                    r#"{"compilerOptions":{"types":[]},"files":["index.ts"]}"#,
                ),
                ("index.ts", "export const x = 1;\n"),
            ],
        );
        let program = crate::program::try_load_version(&format!("{dir}/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let probe = |path: String| {
            std::thread::spawn(move || {
                crate::core::set_thread_program(Some(program));
                crate::program::file_exists(&path)
            })
            .join()
            .unwrap()
        };
        let later = format!("{dir}/later.ts");
        assert!(!probe(later.clone()));
        std::fs::write(&later, "").unwrap();
        let after_write = probe(later);
        let index = probe(format!("{dir}/index.ts"));
        crate::program::release_program(program);
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            !after_write,
            "a worker asked the disk again: the program cache keeps the first answer"
        );
        assert!(index);
    }
}
