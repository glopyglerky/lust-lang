//! Go `internal/project/parsecache.go`.

use crate::project::prelude::*;

use crate::contentmapper;
use crate::frontend::parser;
use crate::xxhash_rust::xxh3::xxh3_128;

// Go: project/parsecache.go:16 ParseCacheKey
// PORT: Go embeds `ast.SourceFileParseOptions` (with its
// `ExternalModuleIndicatorOptions`). The parser structs do not derive `Hash`
// and the plan keeps the parser unchanged, so the key copies their fields:
// `file_name`, `path` and the `jsx` and `force` fields of
// `ExternalModuleIndicatorOptions`. `source_file_parse_options` rebuilds the
// Go embedded value. Go `xxh3.Uint128` is `u128`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ParseCacheKey {
    pub file_name: String,
    pub path: tspath::Path,
    pub jsx: bool,
    pub force: bool,
    pub script_kind: ScriptKind,
    pub hash: u128,
}

impl ParseCacheKey {
    /// Go `key.SourceFileParseOptions` (the embedded value).
    pub fn source_file_parse_options(&self) -> parser::SourceFileParseOptions {
        parser::SourceFileParseOptions {
            file_name: self.file_name.clone(),
            path: self.path.clone(),
            external_module_indicator_options: parser::ExternalModuleIndicatorOptions {
                jsx: self.jsx,
                force: self.force,
            },
        }
    }
}

// Go: project/parsecache.go:22 NewParseCacheKey
// PORT: Go passes the options by value; here by reference.

// Go: project/parsecache.go:39 ContentMappedParseCacheKey (tsgo#4712)
// ContentMappedParseCacheKey identifies the complete output bundle for one mapped input. Hash folds the
// original content, mapper transform identity, and diagnostic locale together.
// PORT: the embedded `ast.SourceFileParseOptions` is copied field by field,
// as in `ParseCacheKey`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ContentMappedParseCacheKey {
    pub file_name: String,
    pub path: tspath::Path,
    pub jsx: bool,
    pub force: bool,
    pub hash: u128,
}

impl ContentMappedParseCacheKey {
    /// Go `ContentMappedParseCacheKey{SourceFileParseOptions: options, Hash: hash}`.
    pub fn new(options: &parser::SourceFileParseOptions, hash: u128) -> ContentMappedParseCacheKey {
        ContentMappedParseCacheKey {
            file_name: options.file_name.clone(),
            path: options.path.clone(),
            jsx: options.external_module_indicator_options.jsx,
            force: options.external_module_indicator_options.force,
            hash,
        }
    }

    /// Go `key.SourceFileParseOptions` (the embedded value).
    pub fn source_file_parse_options(&self) -> parser::SourceFileParseOptions {
        parser::SourceFileParseOptions {
            file_name: self.file_name.clone(),
            path: self.path.clone(),
            external_module_indicator_options: parser::ExternalModuleIndicatorOptions {
                jsx: self.jsx,
                force: self.force,
            },
        }
    }
}

// Go: project/parsecache.go:44 contentMappedParseCacheKey (tsgo#4712)
// PORT: Go `xxh3.Uint128` `Hi` is the high 64 bits of the `u128`, `Lo` the
// low 64 bits.

// Go: project/parsecache.go:55 parseCacheKeyForFile (tsgo#4712)
// parseCacheKeyForFile reconstructs the ordinary parse-cache key for a source file held by a program.

// Go: project/parsecache.go:59 contentMappedParseCacheKeyForFile (tsgo#4712)

// Go: project/parsecache.go:64 parseCacheKeyForDuplicate (tsgo#4712)
// parseCacheKeyForDuplicate reconstructs an ordinary parse-cache key for a deduplicated source file.

// Go: project/parsecache.go:68 contentMappedParseCacheKeyForDuplicate (tsgo#4712)



// Go: project/parsecache.go:72 ParseCache

// Go: project/parsecache.go:74 NewParseCache


// Go: project/parsecache.go:86 ContentMappedParseCache (tsgo#4712)
// PORT: Go embeds `*RefCountCache`; the type alias gives the same methods.
// One reference owns the canonical file and all supplemental files as a
// bundle (`contentmapper::SourceFiles`). Callers ref and deref the
// canonical file only.

// Go: project/parsecache.go:92 NewContentMappedParseCache (tsgo#4712)







