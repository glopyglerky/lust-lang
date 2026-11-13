//! Go `internal/module/resolver.go` lines 1153 to 2361 (unit U19).
//!
//! `resolutionState` methods from `createResolvedModuleHandlingSymlink` to
//! `getTraceFunc`, plus `GetConditions`, `getNodeResolutionFeatures`,
//! `ParsedPatterns`, `TryParsePatterns`, `MatchPatternOrExact`,
//! `normalizePathForCJSResolution`, `extensionIsOk`, `ResolveConfig` and
//! `GetAutomaticTypeDirectiveNames`.
//!
//! PORT: shapes shared with U17 (packagejson) and U18 (resolver_p1, types,
//! cache, util):
//! - Go `*resolved` is `Option<Resolved>`. `nil` (`continueSearching()`) is
//!   `None`. `shouldContinueSearching()` is `is_none()`. `isResolved()` is
//!   `is_some_and(|r| !r.path.is_empty())`.
//! - `resolutionState` is `ResolutionState<'_>` with `resolver: &Resolver`.
//!   Its methods take `&mut self`.
//! - Go `*tracer` is `Option<Rc<RefCell<Tracer>>>`. The resolver shares it
//!   with the state. `trace_write!` keeps the Go `if r.tracer != nil` form.
//! - Go `*packagejson.InfoCacheEntry` is `Option<Rc<InfoCacheEntry>>`.
//!   Go embedding is nesting (`contents.fields.path_fields.exports`).
//! - Go `*ResolvedModule` and `*ResolvedTypeReferenceDirective` results are
//!   owned values. The functions here never return nil for them.
//!
//! PORT: `Ending`, `ResolvedEntrypoint`, `GetEntrypointsFromPackageJsonInfo`,
//! `createResolvedEntrypointHandlingSymlink`, `loadEntrypointsFromExportMap`
//! and `getMatchedStarForPatternEntrypoint` (Go lines 2127 to 2366) are in
//! `entrypoints.rs`. Only `ls/autoimport` and
//! `modulespecifiers.ProcessEntrypointEnding` (language service) use them.

use crate::frontend::prelude::*;
use std::borrow::Cow;

/// Go `if r.tracer != nil { r.tracer.write(diag, args...) }`.
macro_rules! trace_write {
    ($state:expr, $msg:expr $(, $arg:expr)* $(,)?) => {
        if let Some(tracer) = &$state.tracer {
            tracer.borrow_mut().write($msg, args![$($arg),*]);
        }
    };
}

/// Go `(*resolved).isResolved` on a possibly nil `*resolved`.
fn is_resolved(resolved: &Option<Resolved>) -> bool {
    resolved.as_ref().is_some_and(|r| !r.path.is_empty())
}



// Go: module/resolver.go:1911 GetConditions

// Go: module/resolver.go:1934 getNodeResolutionFeatures

// Go: module/resolver.go:1958 moveToNextDirectorySeparatorIfAvailable

// Go: core/pattern.go:5 Pattern
// PORT: Go `core.Pattern` has no crate module in the frontend plan. Only the
// module resolver uses it on the frontend, so it lives here. binder_p1.rs keeps
// its own private copy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pattern {
    pub text: String,
    pub star_index: i32, // -1 for exact match
}

impl Pattern {
    // Go: core/pattern.go:18 IsValid
    pub fn is_valid(&self) -> bool {
        self.star_index == -1 || (self.star_index as usize) < self.text.len()
    }

    // Go: core/pattern.go:22 Matches
    // PORT: Go compares bytes. `text` and `candidate` are port forms, so
    // this compares their Go bytes (see `scanner_util::GO_STRING_MARKER`).
    // `star_index` is the port offset of the star.

    // Go: core/pattern.go:31 MatchedText
    // PORT: Go slices the candidate bytes (see `matches`).

}

// Go: core/pattern.go:10 TryParsePattern

// Go: core/pattern.go:41 FindBestPatternMatch
// PORT: only the `core.Identity` form is used, so this takes the patterns directly.

// Go: module/resolver.go:1971 ParsedPatterns
#[derive(Clone, Debug, Default)]
pub struct ParsedPatterns {
    matchable_string_set: FxHashSet<String>,
    patterns: Vec<Pattern>,
}


// Go: module/resolver.go:1988 TryParsePatterns
// PORT: a nil `*OrderedMap` is `None`.

// Go: module/resolver.go:2023 MatchPatternOrExact

// Go: module/resolver.go:2041 normalizePathForCJSResolution
// If you import from "." inside a containing directory "/foo", the result of `tspath.NormalizePath`
// would be "/foo", but this loses the information that `foo` is a directory and we intended
// to look inside of it. The Node CommonJS resolution algorithm doesn't call this out
// (https://nodejs.org/api/modules.html#all-together), but it seems that module paths ending
// in `.` are actually normalized to `./` before proceeding with the resolution algorithm.

// PORT: the last entry of `get_path_components(path, "")` with no allocation.
// `path` must already have normalized slashes (`combine_paths` output), so the
// combine step in `get_path_components` returns it unchanged. Same rules as
// `path_components`: the root is the first entry, one trailing separator is
// removed, and a path that is only a root (or is empty) ends with the root.

// Go: module/resolver.go:2051 matchesPatternWithTrailer
// PORT: compares the Go bytes of the port forms (see
// `scanner_util::GO_STRING_MARKER`).

// Go: module/resolver.go:2063 extensionIsOk
/** True if `extension` is one of the supported `extensions`. */
pub fn extension_is_ok(extensions: Extensions, extension: &str) -> bool {
    extensions.intersects(Extensions::JAVA_SCRIPT)
        && (extension == EXTENSION_JS
            || extension == EXTENSION_JSX
            || extension == EXTENSION_MJS
            || extension == EXTENSION_CJS)
        || (extensions.intersects(Extensions::TYPE_SCRIPT)
            && (extension == EXTENSION_TS
                || extension == EXTENSION_TSX
                || extension == EXTENSION_MTS
                || extension == EXTENSION_CTS))
        || (extensions.intersects(Extensions::DECLARATION)
            && (extension == EXTENSION_DTS
                || extension == EXTENSION_DMTS
                || extension == EXTENSION_DCTS))
        || (extensions.intersects(Extensions::JSON) && extension == EXTENSION_JSON)
}

// Go: module/resolver.go:2070 ResolveConfig

// Go: module/resolver.go:2075 GetAutomaticTypeDirectiveNames
// PORT: Go returns `[]string{}` for nil `Types`; `unwrap_or_default` does the same.
