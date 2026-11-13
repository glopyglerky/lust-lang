//! The Go `*compiler.Program` methods that implement
//! `ModuleSpecifierGenerationHost` (compiler/program.go), and the parts of
//! the module resolver that they call.

use crate::prelude::*;


use super::deps::OutputPathsHost;
use super::packagejson::{self, };
use super::tspath;
use std::sync::{Arc, PoisonError, RwLock};

/// The current program (`prog()`) as a `ModuleSpecifierGenerationHost`.
/// The program state is reached through the current program, so the host has
/// no fields.
#[derive(Clone, Copy, Debug, Default)]
pub struct ProgramHost;






/// The most programs whose caches a thread keeps. A checker worker serves
/// one program. A thread that serves several (the language server's) keeps
/// the most recently used ones; a program found again after it was dropped
/// fills its caches again, with the same values.
const CACHED_PROGRAMS: usize = 8;




// Go: module/resolver.go:1755 getPackageJsonInfo
// PORT: off the loading thread only (see `ProgramHost`). Tracing is not
// ported. The entries are in the program's `HostFsCache`, not in the
// frontend resolver's cache; the build info reads both
// (`incremental::checker_access::package_json_cache_entries`). The key
// is the `Path` of `<packageDirectory>/package.json`, so another spelling of
// a directory (another case on a case-insensitive file system, or a trailing
// separator) finds the entry of the first spelling, and
// `with_package_directory` gives it this spelling, as in Go.


// Go: module/resolver.go:485 getPackageScopeForPath



#[cfg(test)]
mod tests {
    use super::*;

    // specstat1 skeptic: Go getPackageJsonInfo gives the cached entry the
    // caller's directory (packagejson/cache.go:158 WithPackageDirectory).
    // Two spellings of one directory have one cache key: here a trailing
    // separator, on a case-insensitive file system also another case. The
    // port gave the first spelling's entry as is, so GetPackageJsonInfo
    // (which wants the entry's directory to equal its own) found no
    // package.json, and GetNearestAncestorDirectoryWithPackageJson gave the
    // first spelling.
    #[test]
    fn a_package_json_lookup_keeps_the_spelling_of_its_directory() {
        let dir = std::env::temp_dir().join(format!(
            "goport-package-directory-spelling-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in [
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"types":[]},"files":["index.ts"]}"#,
            ),
            ("index.ts", "export const x = 1;\n"),
            ("node_modules/pkg/package.json", r#"{"name":"pkg"}"#),
        ] {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let dir = dir.to_string_lossy().replace('\\', "/");
        let program = crate::program::try_load_version(&format!("{dir}/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let _scope = crate::core::enter_program(Some(program));
        let pkg = format!("{dir}/node_modules/pkg");
        let thread_pkg = pkg.clone();
        let (with_separator, info, without_separator) = std::thread::spawn(move || {
            crate::core::set_thread_program(Some(program));
            let pkg = thread_pkg;
            (
                ProgramHost.get_nearest_ancestor_directory_with_package_json(&format!("{pkg}/")),
                ProgramHost
                    .get_package_json_info(&format!("{pkg}/package.json"))
                    .map(|entry| entry.package_directory.clone()),
                ProgramHost.get_nearest_ancestor_directory_with_package_json(&pkg),
            )
        })
        .join()
        .unwrap();
        drop(_scope);
        crate::program::release_program(program);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(with_separator, format!("{pkg}/"));
        assert_eq!(info, Some(pkg.clone()));
        assert_eq!(without_separator, pkg);
    }

    // pkgimp1 (editfuzz5 PKG): Go's program finds package.json files with
    // its resolver (compiler/program.go:147-163), whose cache holds the
    // reads of the program load and which reads the program host's file
    // system: in the language server, the snapshot with the open files.
    // The port read the OS file system, so an "imports" map removed in the
    // editor still gave `#m/...` specifiers. Here the program load reads
    // the package.json, which then changes on disk: the loading thread
    // keeps the read of the load, as Go does. Another thread cannot read
    // the frontend resolver and reads the OS file system.
    #[test]
    fn a_loading_thread_lookup_reads_the_package_json_of_the_program() {
        let dir = std::env::temp_dir().join(format!(
            "goport-package-json-program-read-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in [
            (
                "tsconfig.json",
                r#"{"compilerOptions":{"module":"preserve","types":[]},"files":["src/a.ts"]}"#,
            ),
            (
                "package.json",
                r##"{"name":"imp","imports":{"#m/*":"./src/*.ts"}}"##,
            ),
            ("src/a.ts", "export const a = 1;\n"),
        ] {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        let dir = dir.to_string_lossy().replace('\\', "/");
        let program = crate::program::try_load_version(&format!("{dir}/tsconfig.json"), |_| {})
            .unwrap_or_else(|e| panic!("cannot load {dir}: {e}"));
        let _scope = crate::core::enter_program(Some(program));
        std::fs::write(format!("{dir}/package.json"), r#"{"name":"imp"}"#).unwrap();
        let package_json = format!("{dir}/package.json");
        let has_imports = |entry: Option<Arc<InfoCacheEntry>>| {
            entry
                .and_then(|entry| entry.contents.clone())
                .map(|contents| contents.fields.imports.is_present())
        };
        let loading = (
            ProgramHost.get_nearest_ancestor_directory_with_package_json(&format!("{dir}/src")),
            has_imports(ProgramHost.get_package_json_info(&package_json)),
        );
        let thread_package_json = package_json;
        let other = std::thread::spawn(move || {
            crate::core::set_thread_program(Some(program));
            has_imports(ProgramHost.get_package_json_info(&thread_package_json))
        })
        .join()
        .unwrap();
        drop(_scope);
        crate::program::release_program(program);
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(loading, (dir.clone(), Some(true)));
        assert_eq!(other, Some(false));
    }
}
