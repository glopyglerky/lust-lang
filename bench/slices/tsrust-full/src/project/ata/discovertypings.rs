//! Go `internal/project/ata/discovertypings.go`.
//!
//! PORT: Go iterates `map` values here (`inferredTypings`,
//! `possibleSearchDirs`, the dependency maps and the typing cache). Go map
//! order is random. The port keeps insertion order (`IndexMap`) where it
//! owns the map. The order reaches only log text and the order of the
//! returned slices, which `InstallTypings` sorts.

use crate::project::ata::prelude::*;

use crate::frontend::core_ext::TypeAcquisition;
use crate::frontend::core_nodemodules;
use crate::frontend::json::{self, JsonDecoder, JsonError, UnmarshalerFrom, };
use crate::frontend::scanner::scanner_p1::{
    
};
use crate::frontend::{packagejson, semver, tspath, vfs};
use crate::project::logging::{self, Logger as _};

// Go: project/ata/discovertypings.go:20 isTypingUpToDate
// PORT: Go `availableTypingVersions map[string]string` can be nil; `None` is
// the nil map (every lookup misses).

// Go: project/ata/discovertypings.go:29 DiscoverTypings
// PORT: Go `packageNameToTypingLocation *collections.SyncMap` is the
// installer's `RefCell<IndexMap>` (read only here). A `typesRegistry` entry
// is `Option` because Go keeps a JSON `null` entry as a nil map, which the
// `registryEntry != nil` check below tells apart from `{}`.

// Go: project/ata/discovertypings.go:106 addInferredTyping
pub fn add_inferred_typing(inferred_typings: &mut IndexMap<String, String>, typing_name: &str) {
    if !inferred_typings.contains_key(typing_name) {
        inferred_typings.insert(typing_name.to_string(), String::new());
    }
}

// Go: project/ata/discovertypings.go:112 addInferredTypings

// Go: project/ata/discovertypings.go:130 getTypingNamesFromSourceFileNames
// Infer typing names from given file names. For example, the file name "jquery-min.2.3.4.js"
// should be inferred to the 'jquery' typing name; and "angular-route.1.2.3.js" should be inferred
// to the 'angular-route' typing name.
// @param fileNames are the names for source files in the project

// Go: project/ata/discovertypings.go:163 addTypingNamesAndGetFilesToWatch
// Adds inferred typings from manifest/module pairs (think package.json + node_modules)
//
// @param projectRootPath is the path to the directory where to look for package.json, bower.json and other typing information
// @param manifestName is the name of the manifest (package.json or bower.json)
// @param modulesDirName is the directory name for modules (node_modules or bower_components). Should be lowercase!
// @param filesToWatch are the files to watch for changes. We will push things into this array.

// Go: project/ata/discovertypings.go:291 removeMinAndVersionNumbers
// Takes a string like "jquery-min.4.2.3" and returns "jquery"
//
// @internal

// PORT: Go decodes `packagejson.DependencyFields` by reflection with the
// JSON v2 default struct rules (PORTING "JSON"). Only this file decodes that
// struct on its own, so its `UnmarshalerFrom` lives here. The four fields
// are `packagejson.Expected`, which never fails.
