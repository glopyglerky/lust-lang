//! Go `internal/module/resolver.go` lines 1 to 1152 (unit U18).
//!
//! The `resolved` result, the tracer, `resolutionState`, `Resolver` and the
//! `resolutionState` methods from `resolveTypeReferenceDirective` to
//! `loadModuleFromSpecificNodeModulesDirectory`. `resolver_p2.rs` has the rest.
//!
//! PORT: shapes shared with `resolver_p2.rs`:
//! - Go `*resolved` is `Option<Resolved>`. `nil` (`continueSearching()`) is
//!   `None`. `shouldContinueSearching()` is `is_none()`.
//! - Go `*tracer` is `Option<Rc<RefCell<Tracer>>>`. The resolver shares it
//!   with the state. `trace_write!` keeps the Go `if r.tracer != nil` form.
//! - Go `*packagejson.InfoCacheEntry` is `Option<Rc<InfoCacheEntry>>`. Go
//!   embedding is nesting (`contents.fields.path_fields.exports.json_value`).
//! - Go `*core.CompilerOptions` is `Rc<CompilerOptions>`. The state compares
//!   it by pointer with the resolver options.
//! - Go `*ResolvedModule` results are owned values. The resolver caches
//!   hold them as `Rc`.

use crate::frontend::prelude::*;
use crate::gostd::GoError;
use std::sync::Arc;

/// Go `if r.tracer != nil { r.tracer.write(diag, args...) }`.
macro_rules! trace_write {
    ($state:expr, $msg:expr $(, $arg:expr)* $(,)?) => {
        if let Some(tracer) = &$state.tracer {
            tracer.borrow_mut().write($msg, args![$($arg),*]);
        }
    };
}

/// Go `(*resolved).isResolved` on a possibly nil `*resolved`.
fn resolved_is_resolved(resolved: &Option<Resolved>) -> bool {
    resolved.as_ref().is_some_and(|r| !r.path.is_empty())
}


// Go: module/resolver.go:19 resolved
#[derive(Clone, Debug, Default)]
pub struct Resolved {
    pub path: String,
    pub extension: String,
    pub package_id: PackageId,
    pub original_path: String,
    pub resolved_using_ts_extension: bool,
    // tsgo#4712
    pub resolved_using_extra_extensions: bool,
}

// Go: module/resolver.go:28 resolved.shouldContinueSearching
// PORT: `Option::is_none` on `Option<Resolved>`.

// Go: module/resolver.go:32 resolved.isResolved
// PORT: the private `resolved_is_resolved` helper.

// Go: module/resolver.go:36 continueSearching
#[must_use]
pub fn continue_searching() -> Option<Resolved> {
    None
}

// Go: module/resolver.go:40 unresolved
#[must_use]
pub fn unresolved() -> Option<Resolved> {
    Some(Resolved::default())
}

// Go: module/resolver.go:44 resolutionKindSpecificLoader
// PORT: Go closures capture `r`. Here the loader gets the state as its first
// argument, so the caller can keep `&mut self`.

// Go: module/resolver.go:46 tracer

// Go: module/resolver.go:50 DiagAndArgs
// PORT: Go `Args []any` is `Vec<String>`. Values are formatted with
// `ToString` when written (`args!`), like the Go `%v` formatting later.



// Go: module/resolver.go:68 resolutionState


// Go: module/resolver.go:93 newResolutionState

// Go: module/resolver.go:138 GetCompilerOptionsWithRedirect

// Go: module/resolver.go:148 DefaultResolver
// PORT: Go embeds `caches`; here it is the `caches` field.

// Go: module/resolver.go:158 ResolverOptions
// PORT: the Go nil `Host` and `CompilerOptions` are `None`. Go copies the
// struct by value; this is `Clone`.

// Go: module/resolver.go:167 NewResolver
// PORT: Go keeps a nil `Host` or `CompilerOptions` and panics at the first
// use; `DefaultResolver` has no nil for them, so this panics now.





