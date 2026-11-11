//! Explicit expansion interface for Cargo's externally supplied crate name.
//! The inherited single-file entry point derives it from the source filename.
//! Reuse the original expander, passing its existing public context field.

use crate::upstream::expand as upstream;
pub(crate) use upstream::*;

pub fn expand_lexed_with_deps(
    lexed: crate::lexer::Lexed,
    ctx: &ExpandCtx,
) -> Result<(crate::lexer::Lexed, Vec<Dep>), crate::diag::Diag> {
    let Some(name) = crate::cargo_cli::crate_name(&ctx.real_path).filter(|_| ctx.crate_name.is_none()) else {
        return upstream::expand_lexed_with_deps(lexed, ctx);
    };
    let ctx = ExpandCtx {
        interner: ctx.interner, known: ctx.known, file_path: ctx.file_path.clone(),
        real_path: ctx.real_path.clone(), cargo_env: ctx.cargo_env.clone(), src: ctx.src,
        cfg: builtin::CfgEnv { test: ctx.cfg.test, debug_assertions: ctx.cfg.debug_assertions,
            facts: ctx.cfg.facts.clone(), panic_unwind: ctx.cfg.panic_unwind },
        module_path: ctx.module_path.clone(),
        crate_name: Some(name.into()), extern_crates: ctx.extern_crates.clone(), client: ctx.client,
    };
    upstream::expand_lexed_with_deps(lexed, &ctx)
}
