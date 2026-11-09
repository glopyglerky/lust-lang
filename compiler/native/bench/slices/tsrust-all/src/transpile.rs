//! Go package `transpile` (#4849): single-file JavaScript and declaration
//! emit.
//!
//! PORT: Go makes a program of one file over a `transpileFS` file system
//! (fs.go, ts#64009) and emits it. Here that program is a program version of the
//! process (`program::new_program_version`, as the compiler runner makes
//! its programs). It is read inside `core::enter_program`, and its checker
//! and emit pools are freed after the emit (`program::release_program`).
//! Its file versions stay leaked, like those of every program version, so
//! the file nodes of the result diagnostics stay valid.

use crate::prelude::*;

use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use crate::emitter::emitter::EmitOnly;
use crate::emitter::program_emit::{self, };
use crate::frontend::compiler::{};
use crate::frontend::tsoptions::{};
use crate::frontend::tspath::{};
use crate::frontend::vfs::{Entries, FileInfo, FsError};
use crate::program;

// Go: transpile/transpile.go:17 Options

// Go: transpile/transpile.go:39 Output

// Go: transpile/transpile.go:47 inputDirectory
// inputDirectory is the synthetic current directory used to root the
// single input file created for transpilation.
const INPUT_DIRECTORY: &str = "/";

// Go: transpile/transpile.go:51 libDirectory
// libDirectory is the synthetic directory that the barebones default library
// file is placed in for declaration transpilation. See [barebonesLibContent].
const LIB_DIRECTORY: &str = "/lib";

// Go: transpile/transpile.go:59 barebonesLibContent
// Declaration emit works without a `lib`, but some local inferences you'd
// expect to work won't without at least a minimal `lib` available, since the
// checker will type inferred declarations as `any` without these defined.
// Late bound symbol names, in particular, are impossible to define without
// `Symbol` at least partially defined.
// TODO: This should *probably* just load the full, real `lib` for the target.
const BAREBONES_LIB_CONTENT: &str = r"interface Boolean {}
interface Function {}
interface CallableFunction {}
interface NewableFunction {}
interface IArguments {}
interface Number {}
interface Object {}
interface RegExp {}
interface String {}
interface Array<T> { length: number; [n: number]: T; }
interface SymbolConstructor {
    (desc?: string | number): symbol;
    for(name: string): symbol;
    readonly toStringTag: symbol;
}
declare var Symbol: SymbolConstructor;
interface Symbol {
    readonly [Symbol.toStringTag]: string;
}";

// Go: transpile/transpile.go:93 TranspileModule

// Go: transpile/transpile.go:114 TranspileDeclaration

/// The texts that the emit writes (Go `outputText`, `hasOutputText`,
/// `sourceMapText` and `hasSourceMapText`). `None` is "not written".
#[derive(Default)]
struct Written {
    output_text: Option<String>,
    source_map_text: Option<String>,
}

// Go: transpile/transpile.go:118 transpileWorker

// Go: transpile/fs.go:11 transpileFS
// transpileFS embeds unsupported operations so unexpected filesystem access
// panics.
// PORT: Go embeds a nil `vfs.FS`, so any other method dereferences nil
// (`go_nil_dereference`).
struct TranspileFs {
    files: FxHashMap<String, String>,
}


#[cfg(test)]
mod tests {
    // Go: transpile/fs_test.go
    use super::*;

    use std::panic::{AssertUnwindSafe, catch_unwind};

    // Go: internal/testutil/testutil.go:14 AssertPanics
    fn assert_panics(f: impl FnOnce(), expected: &str) {
        let payload = catch_unwind(AssertUnwindSafe(f)).expect_err("expected a panic");
        let got = payload
            .downcast_ref::<GoPanic>()
            .map(|p| p.message.clone())
            .unwrap_or_else(|| panic!("expected a Go panic with {expected:?}"));
        assert_eq!(got, expected);
    }

    // Go: fs_test.go:9 TestTranspileFSRejectsDirectoryAccess
    #[test]
    fn test_transpile_fs_rejects_directory_access() {
        let mut files = FxHashMap::default();
        files.insert("/src/module.ts".to_string(), String::new());
        let fs = TranspileFs { files };
        assert_panics(
            || {
                fs.directory_exists("/src");
            },
            r#"unexpected directory existence check for "/src""#,
        );
        assert_panics(
            || {
                fs.realpath("/src/module.ts");
            },
            r#"unexpected realpath request for "/src/module.ts""#,
        );
    }
}
