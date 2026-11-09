//! Port of Go `transformers/moduletransforms/externalmoduleinfo.go`.

use crate::prelude::*;
use crate::transformers::transformer::TransformReferenceResolver;

/// Go `collections.MultiMap[K, V]`.
// PORT: Go never iterates these maps, so a hash map keeps the Go behavior.
#[derive(Debug)]
pub(crate) struct MultiMap<K, V> {
    m: FxHashMap<K, Vec<V>>,
}

impl<K, V> Default for MultiMap<K, V> {
    fn default() -> Self {
        Self {
            m: FxHashMap::default(),
        }
    }
}

impl<K: std::hash::Hash + Eq, V: Clone> MultiMap<K, V> {
    /// Go `MultiMap.Add`.
    pub(crate) fn add(&mut self, key: K, value: V) {
        self.m.entry(key).or_default().push(value);
    }

    /// Go `MultiMap.Get`. A missing key gives an empty (Go nil) slice.
    pub(crate) fn get<Q>(&self, key: &Q) -> Vec<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.m.get(key).cloned().unwrap_or_default()
    }

    /// Go `MultiMap.Has`.
    pub(crate) fn has<Q>(&self, key: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq + ?Sized,
    {
        self.m.contains_key(key)
    }

    /// Go `MultiMap.Len`.
    pub(crate) fn len(&self) -> usize {
        self.m.len()
    }
}

// Go: transformers/moduletransforms/externalmoduleinfo.go:15 externalModuleInfo
#[derive(Debug, Default)]
pub(crate) struct ExternalModuleInfo {
    /// ImportDeclaration | ImportEqualsDeclaration | ExportDeclaration. imports and reexports of other external modules
    pub external_imports: Vec<Node>,
    /// Maps local names to their associated export specifiers (excludes reexports)
    pub export_specifiers: MultiMap<String, Node>,
    /// Maps local declarations to their associated export aliases
    pub exported_bindings: MultiMap<Node, Node>,
    /// all exported names in the module, both local and re-exported, excluding the names of locally exported function declarations
    pub exported_names: Vec<Node>,
    /// all of the top-level exported function declarations
    pub exported_functions: IndexSet<Node>,
    /// an export=/module.exports= declaration if one was present
    pub export_equals: Node,
    /// whether this module contains export*
    pub has_export_stars_to_export_values: bool,
}

// Go: transformers/moduletransforms/externalmoduleinfo.go:25 externalModuleInfoCollector

// Go: transformers/moduletransforms/externalmoduleinfo.go:35 collectExternalModuleInfo


const EXTERNAL_HELPERS_MODULE_NAME_TEXT: &str = "tslib";

// Go: transformers/moduletransforms/externalmoduleinfo.go:262 createExternalHelpersImportDeclarationIfNeeded

// Go: transformers/moduletransforms/externalmoduleinfo.go:320 getImportedHelpers

// Go: transformers/moduletransforms/externalmoduleinfo.go:330 getOrCreateExternalHelpersModuleNameIfNeeded

// Go: transformers/moduletransforms/externalmoduleinfo.go:348 isNamedDefaultReference

// Go: transformers/moduletransforms/externalmoduleinfo.go:352 containsDefaultReference

// Go: transformers/moduletransforms/externalmoduleinfo.go:356 getExportNeedsImportStarHelper

// Go: transformers/moduletransforms/externalmoduleinfo.go:360 getImportNeedsImportStarHelper

// Go: transformers/moduletransforms/externalmoduleinfo.go:385 getImportNeedsImportDefaultHelper
