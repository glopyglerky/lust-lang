//! Go `internal/project/configfileregistry.go`.
//!
//! PORT: one thread (project/dirty/interfaces.rs). Go `*ConfigFileRegistry`
//! is `Rc<ConfigFileRegistry>` (never changed after it is shared). Go
//! `*configFileEntry` and `*configFileNames` are `Rc<RefCell<..>>`: the
//! registry builder changes them through `dirty` entries. Go
//! `map[tspath.Path]struct{}` is `FxHashSet<tspath::Path>` and a nil map is
//! the empty set (Go only reads nil maps and makes one before a write).

use crate::project::prelude::*;

// Go: project/configfileregistry.go:15 ConfigFileRegistry

// Go: project/configfileregistry.go:28 configuredContentMappers (tsgo#4712)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfiguredContentMappers {
    pub extensions: Vec<String>,
}

// Go: project/configfileregistry.go:32 collectConfiguredContentMappers (tsgo#4712)

// Go: project/configfileregistry.go:61 configFileEntry

// Go: project/configfileregistry.go:88 newConfigFileEntry

// Go: project/configfileregistry.go:101 newExtendedConfigFileEntry


// Go: project/configfileregistry.go:109 configFileEntry.Clone (dirty.Cloneable)


// Go: project/configfileregistry.go:160 TestConfigEntry
// For testing
// PORT: Go `iter.Seq[tspath.Path]` over a live map is the collected keys.
#[derive(Clone, Debug, Default)]
pub struct TestConfigEntry {
    pub file_name: String,
    pub retaining_projects: Vec<ID>,
    pub retaining_open_files: Vec<tspath::Path>,
    pub retaining_configs: Vec<tspath::Path>,
}

// Go: project/configfileregistry.go:196 TestConfigFileNamesEntry
// PORT: Go shares the entry's `ancestors` map; the port copies it.
#[derive(Clone, Debug, Default)]
pub struct TestConfigFileNamesEntry {
    pub nearest_config_file_name: String,
    pub ancestors: FxHashMap<String, String>,
}

// Go: project/configfileregistry.go:226 configFileNames
#[derive(Clone, Debug, Default)]
pub struct ConfigFileNames {
    // nearestConfigFileName is the file name of the nearest ancestor config file.
    pub nearest_config_file_name: String,
    // ancestors is a map from one ancestor config file path to the next.
    // For example, if `/a`, `/a/b`, and `/a/b/c` all contain config files,
    // the fully loaded map will look like:
    //		{
    //			"/a/b/c/tsconfig.json": "/a/b/tsconfig.json",
    //			"/a/b/tsconfig.json": "/a/tsconfig.json"
    //		}
    // PORT: a nil map is the empty map.
    pub ancestors: FxHashMap<String, String>,
}

impl ConfigFileNames {
    // Go: project/configfileregistry.go:239 configFileNames.Clone
    // PORT: Go `Clone()` is `clone_` (dirty decision 3).
    pub fn clone_(&self) -> Rc<RefCell<ConfigFileNames>> {
        Rc::new(RefCell::new(ConfigFileNames {
            nearest_config_file_name: self.nearest_config_file_name.clone(),
            ancestors: self.ancestors.clone(),
        }))
    }
}

// Go: project/configfileregistry.go:239 configFileNames.Clone (dirty.Cloneable)
impl dirty::Cloneable for Rc<RefCell<ConfigFileNames>> {
    fn clone_(&self) -> Self {
        self.borrow().clone_()
    }
}
