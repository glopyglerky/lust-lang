//! Port of module/types.go.

use crate::frontend::prelude::*;
use std::sync::Arc;

// Go: module/types.go:14 ResolutionHost
// PORT: Go interface. `FS()` returns a borrowed trait object; the host
// keeps the shared `Rc<dyn Fs>`.

// Go: module/types.go:20 Resolver (ts#64299)
// PORT: Go interface. A nil `*ResolvedModule` is `None`; a nil `error` is
// `None`. `PackageJsonCacheEntries` takes a `dyn FnMut`, because a trait
// object has no generic methods.

// Go: module/resolver.go:148 DefaultResolver as a module.Resolver
// PORT: each method is the `DefaultResolver` method of the same Go name.

// Go: module/types.go:19 ModeAwareCacheKey
// PORT: the derived `Hash` must stay the one of `dyn ModeAwareKey` (name,
// then mode).
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ModeAwareCacheKey {
    pub name: String,
    pub mode: ResolutionMode,
}

/// A `ModeAwareCache` key by its parts. `cache.get(&(name, mode) as &dyn
/// ModeAwareKey)` looks up a borrowed name with no owned key. No Go
/// counterpart.
pub trait ModeAwareKey {
    fn parts(&self) -> (&str, ResolutionMode);
}

impl ModeAwareKey for ModeAwareCacheKey {
    fn parts(&self) -> (&str, ResolutionMode) {
        (&self.name, self.mode)
    }
}

impl ModeAwareKey for (&str, ResolutionMode) {
    fn parts(&self) -> (&str, ResolutionMode) {
        *self
    }
}





// Go: module/types.go:24 ResolvedProjectReference
// PORT: Go `module.ResolvedProjectReference` is an interface. The name
// `ResolvedProjectReference` is already the program.rs struct, so the
// interface is `ModuleResolvedProjectReference`. Go returns a nil
// `*core.CompilerOptions` as `None`. The options are shared (`Rc`), because
// the resolution state compares them by pointer with the resolver options.

// Go: module/types.go:29 NodeResolutionFeatures
crate::flags_macros::go_flags!(NodeResolutionFeatures, i32 {
    IMPORTS = 1 << 0; // NodeResolutionFeaturesImports
    SELF_NAME = 1 << 1; // NodeResolutionFeaturesSelfName
    EXPORTS = 1 << 2; // NodeResolutionFeaturesExports
    EXPORTS_PATTERN_TRAILERS = 1 << 3; // NodeResolutionFeaturesExportsPatternTrailers
    // allowing `#/` root imports in package.json imports field
    // not supported until mass adoption - https://github.com/nodejs/node/pull/60864
    IMPORTS_PATTERN_ROOT = 1 << 4; // NodeResolutionFeaturesImportsPatternRoot

    NONE = 0; // NodeResolutionFeaturesNone
    ALL = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4); // NodeResolutionFeaturesAll
    NODE16_DEFAULT = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3); // NodeResolutionFeaturesNode16Default
    NODE_NEXT_DEFAULT = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4); // NodeResolutionFeaturesNodeNextDefault
    BUNDLER_DEFAULT = (1 << 0) | (1 << 1) | (1 << 2) | (1 << 3) | (1 << 4); // NodeResolutionFeaturesBundlerDefault
});

// Go: module/types.go:47 PackageId
// PORT: the struct is `program::PackageId`. Its Go methods are here.
impl PackageId {
    // Go: module/types.go:54 PackageId.String
    #[must_use]
    pub fn string(&self) -> String {
        format!(
            "{}@{}{}",
            self.package_name(),
            self.version,
            self.peer_dependencies
        )
    }

    // Go: module/types.go:58 PackageId.PackageName
    #[must_use]
    pub fn package_name(&self) -> String {
        if !self.sub_module_name.is_empty() {
            return format!("{}/{}", self.name, self.sub_module_name);
        }
        self.name.clone()
    }
}

// Go: module/types.go:65 ResolvedModule
// PORT: the struct and `IsResolved` are `program::ResolvedModule`.

// Go: module/types.go:80 ResolvedTypeReferenceDirective


// Go: module/types.go:93 extensions
// PORT: Go unexported `extensions`. Other files of the package use it, so
// it is `pub`.
crate::flags_macros::go_flags!(Extensions, i32 {
    TYPE_SCRIPT = 1 << 0; // extensionsTypeScript
    JAVA_SCRIPT = 1 << 1; // extensionsJavaScript
    DECLARATION = 1 << 2; // extensionsDeclaration
    JSON = 1 << 3; // extensionsJson

    IMPLEMENTATION_FILES = (1 << 0) | (1 << 1); // extensionsImplementationFiles
});

impl Extensions {
    // Go: module/types.go:104 extensions.String

    // Go: module/types.go:121 extensions.Array
    #[must_use]
    pub fn array(self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        if self.intersects(Extensions::TYPE_SCRIPT) {
            result.extend(
                SUPPORTED_TS_IMPLEMENTATION_EXTENSIONS
                    .iter()
                    .map(|e| (*e).to_string()),
            );
        }
        if self.intersects(Extensions::JAVA_SCRIPT) {
            result.extend(
                SUPPORTED_JS_EXTENSIONS_FLAT
                    .iter()
                    .map(|e| (*e).to_string()),
            );
        }
        if self.intersects(Extensions::DECLARATION) {
            result.extend(
                SUPPORTED_DECLARATION_EXTENSIONS
                    .iter()
                    .map(|e| (*e).to_string()),
            );
        }
        if self.intersects(Extensions::JSON) {
            result.push(EXTENSION_JSON.to_string());
        }
        result
    }
}
