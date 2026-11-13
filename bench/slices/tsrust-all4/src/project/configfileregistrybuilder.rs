//! Go `internal/project/configfileregistrybuilder.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). The builder is shared as
//! `Rc<ConfigFileRegistryBuilder>` (Go `*configFileRegistryBuilder`). Go
//! passes the builder itself as the `tsoptions.ParseConfigHost` and
//! `tsoptions.ExtendedConfigCache` of the extended config cache arguments,
//! which own their values, so the builder keeps a `Weak` to itself
//! (`this`). `dirty` values are `Rc<RefCell<..>>` handles; `change` and
//! `change_if` give the handle to `apply`, which changes it through
//! `borrow_mut`. Go `map[tspath.Path]struct{}` is `FxHashSet<tspath::Path>`;
//! a local Go map that can stay nil is an `Option`.

use crate::project::prelude::*;

use crate::frontend::vfs::Fs as _;
use std::cell::Cell;
use std::rc::Weak;

// Go: project/configfileregistrybuilder.go:18 interface assertions
// PORT: the `tsoptions::ParseConfigHost` and `tsoptions::ExtendedConfigCache`
// impls at the end of this file.

// Go: project/configfileregistrybuilder.go:27 configFileRegistryBuilder
// configFileRegistryBuilder tracks changes made on top of a previous
// configFileRegistry, producing a new clone with `finalize()` after
// all changes have been made.

// Go: project/configfileregistrybuilder.go:44 newConfigFileRegistryBuilder
// PORT: the unused Go `logger` parameter is kept. The base maps are copied
// into the dirty maps (project/dirty/interfaces.rs decision 3).


// Go: project/configfileregistrybuilder.go:656 contentMapperManifestPath (tsgo#4712)

// Go: project/configfileregistrybuilder.go:395 changeFileResult
// PORT: a nil Go map is the empty set.
#[derive(Clone, Debug, Default)]
pub struct ChangeFileResult {
    // ts#64319: project IDs.
    pub affected_projects: FxHashSet<ID>,
    pub affected_files: FxHashSet<tspath::Path>,
}

impl ChangeFileResult {
    // Go: project/configfileregistrybuilder.go:400 changeFileResult.IsEmpty
    pub fn is_empty(&self) -> bool {
        self.affected_projects.is_empty() && self.affected_files.is_empty()
    }
}

// Go: project/configfileregistrybuilder.go:19 `_ tsoptions.ParseConfigHost = (*configFileRegistryBuilder)(nil)`

// Go: project/configfileregistrybuilder.go:20 `_ tsoptions.ExtendedConfigCache = (*configFileRegistryBuilder)(nil)`
