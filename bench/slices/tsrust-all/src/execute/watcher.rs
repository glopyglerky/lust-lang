//! Go: execute/watcher.go (package `execute`): `tsc --watch` without
//! `--build`.
//!
//! PORT: testing. Go `testing tsc.CommandLineTesting` is `None` outside
//! tests (see tsc/compile.rs). The test watch backend comes from
//! `set_test_watch_backend`, not from `testing`.
//!
//! PORT: Go runs `DoCycle` from `WatchManager.RunLoop` on the goroutine
//! that called `start`. The port does the same on the calling thread. The
//! watch manager is `Rc<RefCell<WatchManager>>`: the loop keeps a shared
//! borrow while `do_cycle` borrows the watcher mutably, and `do_cycle` only
//! takes shared borrows of the manager.
//!
//! PORT: Go makes a new program on every build. Each build here makes a
//! program version (`execute_tsc::new_frontend_program`) and runs with it
//! current (`core::enter_program`). Files that the source file cache keeps
//! are shared with the last version. The last version is released when the
//! next build has read it (Go drops the old program there). The rest of the
//! old program is freed after the next build has reported its status.

use crate::contentmapper::{self, Mapper, };
use crate::execute::execute_tsc::{};
use crate::execute::incremental;
use crate::execute::tsc::compile::{
    
    
};
use crate::execute::tsc::diagnostics::{
    
};
use crate::execute::tsc::emit::{};
use crate::execute::watchmanager::{
    new_dir_watch_set, 
};
use crate::frontend::prelude::*;
use crate::frontend::vfs::trackingvfs;
use crate::fswatch;
use crate::gostd::{GoError};
use std::sync::Arc;
use std::time::SystemTime;

// Go: execute/watcher.go:26 cachedSourceFile
// PORT: Go `time.Time` is `Option<SystemTime>` (`None` = zero), as in
// `vfs::FileInfo`.

// Go: execute/watcher.go:31 watchCompilerHost
// PORT: the embedded Go `compiler.CompilerHost` is the `compiler_host`
// field; the trait impl below forwards to it. Go `*collections.SyncMap`
// shared with the watcher is `Rc<RefCell<FxHashMap>>` (one thread).




// Go: execute/watcher.go:59 Watcher
// PORT: Go `*tsoptions.ParsedCommandLine` is `Rc<ParsedCommandLine>`. Go
// `*collections.OrderedMap[string, any]` is
// `Option<IndexMap<String, CompilerOptionsValue>>` (as in build/host.rs).
// Go `*incremental.Program` and `*tsc.ExtendedConfigCache` are `Option`s
// (nil before `start`). Go `*collections.Set` of seen files is a plain
// set (empty before the first build, which Go's nil set reads as). Go
// `time.Time` is `Option<SystemTime>`.

// Go: execute/watcher.go:103 `var _ tsc.Watcher = (*Watcher)(nil)`




// Go: execute/watcher.go:105 createWatcher


// Go: execute/watcher.go:581 equalJSXImplicitImport
// PORT: the files are frontend `ParsedSourceFile`s (the new one is not
// published), so the base comes from the file loader's
// `get_jsx_implicit_import_base_of_file`, Go `ast.GetJSXImplicitImportBase`
// on the parser's pragmas.
