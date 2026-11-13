// rustc E0275 at an impl's well-formedness check: each where clause of a
// written impl is proved under the impl's own param env, and when an impl
// header matches it as well as the env, rustc evaluates that impl's
// predicates too (traits::solve::eval). A where clause that asks the
// same trait at a growing type overflows there, whether or not anything
// uses the impl.
use crate::check::collect::Program;
use crate::diag::{Diag, Span};
use crate::traits::solve::{PureCx, Solver, eval::Evaluator};
use crate::ty::*;

/// The crate's `#![recursion_limit = "N"]`, else rustc's default 128.
/// expand rejects a malformed value and one below 128. Every body check
/// asks, so a program whose source never names the attribute (its name
/// was never interned) answers without the scan of every inner attribute.
pub fn recursion_limit(p: &Program) -> usize {
    let Some(name) = p.interner.find("recursion_limit") else { return 128 };
    for a in &p.ast.inner_attrs {
        if a.path.len() != 1 || a.path[0].name != name {
            continue;
        }
        // `= "N"` arrives as an expression or, after expansion, as the
        // raw tokens.
        let text = match &a.input {
            crate::ast::AttrInput::Eq(e) => match p.ast.expr(*e) {
                crate::ast::Expr::Str(i) => p.ast.strings.get(*i as usize),
                _ => None,
            },
            crate::ast::AttrInput::List(_, trees) => trees.iter().find_map(|t| match t {
                crate::lexer::TokenTree::Token(tok) => match tok.tok {
                    crate::lexer::Tok::Str(i, _) => p.ast.strings.get(i as usize),
                    _ => None,
                },
                _ => None,
            }),
            crate::ast::AttrInput::None => None,
        };
        if let Some(n) = text.and_then(|s| s.parse::<usize>().ok()) {
            return n;
        }
    }
    128
}

/// Reports every written impl with a where clause whose evaluation passes
/// the recursion limit (L0611, rustc E0275).
pub fn check_impls(p: &mut Program, errs: &mut Vec<Diag>) {
    use crate::traits::index::Synthetic;
    let limit = recursion_limit(p);
    let lang_sized = p.gx.lang.sized;
    let records: Vec<(Span, Vec<Predicate>)> =
        p.gx.index
            .all_impls()
            .filter(|(_, r)| r.synthetic == Synthetic::None && r.local)
            .filter(|(_, r)| {
                r.generics.predicates.iter().any(|q| matches!(q, Predicate::Trait(b) if b.count == 0 && b.value.def != lang_sized))
            })
            .map(|(_, r)| (r.span, r.generics.predicates.clone()))
            .collect();
    for (span, preds) in records {
        let env = crate::check::generics::elaborate_env(&mut p.tys, &p.gx.index, &preds);
        let mut cx = PureCx::new(&p.tys);
        let mut solver = Solver::new();
        let mut ev = Evaluator::new(limit);
        let over = preds.iter().find_map(|q| match q {
            Predicate::Trait(b) if b.count == 0 && b.value.def != lang_sized => {
                ev.where_clause_overflows(&mut solver, &mut cx, &p.gx.index, &env, b.value).then_some(b.value.def)
            }
            _ => None,
        });
        if let Some(def) = over {
            let tname = p.gx.index.trait_record(def).map_or_else(|| "?".to_string(), |r| p.display_path(r.name));
            errs.push(Diag::new(
                "L0611",
                span,
                format!("overflow evaluating a `{tname}` where clause of this impl: the impls it needs recurse past the recursion limit"),
                "Remove the where clause that asks the trait at a growing type.",
            ));
        }
    }
}
