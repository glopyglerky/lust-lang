//! Go `internal/project/autoimport.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go
//! `collections.SyncMap` is a `RefCell<FxHashMap>`; `filesMu` is dropped.
//! Go `FileHandle` is `Option<Rc<dyn FileHandle>>` (nil is `None`). In this
//! package's prelude, `autoimport` is Go package `ls/autoimport`.

use crate::project::prelude::*;

use crate::frontend::module;
use crate::frontend::parser;

// Go: project/autoimport.go:16 autoImportBuilderFS
// PORT: the untracked map stores Go nil handles too (Go `LoadOrStore` of a
// nil `fh` caches the miss), so its values are `Option`.

// Go: project/autoimport.go:20 `var _ FileSource = (*autoImportBuilderFS)(nil)`


/// The parse cache keys that auto-import registry clones acquired, one per
/// path, kept after each clone. The session owns it
/// (`Session::auto_import_parse_keys`). See
/// `AutoImportRegistryCloneHost::dispose`.
// PORT: no Go counterpart.
pub type AutoImportParseKeys = RefCell<FxHashMap<tspath::Path, ParseCacheKey>>;

// Go: project/autoimport.go:61 autoImportRegistryCloneHost
// PORT: `filesMu` is dropped; `files` is written after sharing, so it is a
// `RefCell`. `kept_files` has no Go counterpart (see `dispose`).

// Go: project/autoimport.go:73 newAutoImportRegistryCloneHost
// PORT: `kept_files` is the session's `auto_import_parse_keys`.

// PORT: Go `autoimport.RegistryCloneHost` embeds `module.ResolutionHost`;
// its `FS` and `GetCurrentDirectory` are this supertrait.

// Go: project/autoimport.go:77 `var _ autoimport.RegistryCloneHost = (*autoImportRegistryCloneHost)(nil)`
