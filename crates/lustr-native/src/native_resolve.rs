//! Explicit resolver interface for Cargo's externally supplied crate name.
//! No resolver implementation is copied or patched. Only the root request's
//! public CrateInput is adjusted; std and dependency crate identities remain
//! those constructed by the original resolver.

use crate::upstream::resolve as upstream;
pub(crate) use upstream::*;

pub mod worklist {
    pub use super::upstream::worklist::*;

    pub fn resolve_crate_with(
        input: &CrateInput,
        interner: &crate::intern::Interner,
        known: &crate::intern::Known,
        cache: &mut ResolveCache,
        with_platform: bool,
    ) -> Result<ResolvedCrate, ResolveError> {
        let Some(name) = crate::cargo_cli::crate_name(&input.root_path) else {
            return super::upstream::worklist::resolve_crate_with(input, interner, known, cache, with_platform);
        };
        let input = CrateInput { crate_name: name.into(), ..input.clone() };
        super::upstream::worklist::resolve_crate_with(&input, interner, known, cache, with_platform)
    }
}
