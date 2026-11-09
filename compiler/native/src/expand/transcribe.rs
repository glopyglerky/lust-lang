// Transcription (ARCH 6.2): substitute metavariables and repetitions
// into a matched arm's template. Tokens the template writes get the
// expansion's context; tokens that come from a metavariable keep the
// spans and contexts they arrived with (ARCH 6.3).
use super::match_::RepOp;
use super::*;
use crate::diag::CtxtId;

/// Transcribes `template` with `binds`, stamping template tokens with
/// context `ctxt`. `call_span` anchors errors and synthetic tokens.
pub fn transcribe(template: &[Tt], binds: &BindMap, ctxt: CtxtId, call_span: Span, interner: &Interner) -> Result<Vec<Tt>, Diag> {
    let mut tx = Tx { binds, ctxt, call_span, interner, budget: 8_000_000 };
    let mut indices = Vec::new();
    tx.seq(template, &mut indices)
}

struct Tx<'a> {
    binds: &'a BindMap,
    ctxt: CtxtId,
    call_span: Span,
    interner: &'a Interner,
    budget: u32,
}

impl<'a> Tx<'a> {
    fn step(&mut self) -> Result<(), Diag> {
        self.budget = self.budget.saturating_sub(1);
        if self.budget == 0 {
            return Err(err(C_CAP, self.call_span, "macro expansion exceeded its token limit", "Simplify the macro or the call."));
        }
        Ok(())
    }

    /// This span with the expansion's context, for template tokens.
    fn stamp(&self, span: Span) -> Span {
        Span { ctxt: self.ctxt, ..span }
    }

    /// Transcribes one sequence of template trees.
    fn seq(&mut self, trees: &[Tt], indices: &mut Vec<usize>) -> Result<Vec<Tt>, Diag> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < trees.len() {
            self.step()?;
            match &trees[i] {
                Tt::Tok(t) if t.tok == Tok::Dollar => {
                    let span = t.span;
                    match trees.get(i + 1) {
                        // A repetition.
                        Some(Tt::Delim(Delim::Paren, inner, _)) => {
                            let inner = inner.clone();
                            let (sep, op, used) = super::match_::rep_tail(trees, i + 2, true).ok_or_else(|| {
                                err(C_DEF, span, "repetition has no operator", "Write `$(...)*`, `$(...)+`, or `$(...)?`.")
                            })?;
                            if sep.is_some() && op == RepOp::Question {
                                return Err(err(
                                    C_DEF,
                                    span,
                                    "the `?` repetition operator does not take a separator",
                                    "Drop the separator, or use `*` for a separated repetition.",
                                ));
                            }
                            self.repetition(&inner, sep, op, span, indices, &mut out)?;
                            i += 2 + used;
                        }
                        Some(Tt::Tok(name_tok)) if name_tok.tok == Tok::Dollar => {
                            // `$$` is the unstable metavariable escape;
                            // rustc rejects it on stable.
                            return Err(err(
                                C_UNSUPPORTED,
                                span,
                                "`$$` metavariable expressions are not supported",
                                "Capture a `$` as a tt argument and forward it instead.",
                            ));
                        }
                        Some(Tt::Delim(Delim::Brace, _, _)) => {
                            return Err(err(
                                C_UNSUPPORTED,
                                span,
                                "`${..}` metavariable expressions are not supported",
                                "Compute the value with a helper macro instead.",
                            ));
                        }
                        Some(Tt::Tok(name_tok)) => {
                            i = self.metavar(*name_tok, span, i, indices, &mut out)?;
                        }
                        _ => {
                            // A lone `$` is a literal token in rustc's
                            // transcriber, not an error.
                            out.push(Tt::Tok(Token { tok: Tok::Dollar, span: self.stamp(span) }));
                            i += 1;
                        }
                    }
                }
                Tt::Tok(t) => {
                    out.push(Tt::Tok(Token { tok: t.tok, span: self.stamp(t.span) }));
                    i += 1;
                }
                Tt::Delim(d, inner, span) => {
                    let inner = inner.clone();
                    let done = self.seq(&inner, indices)?;
                    out.push(Tt::Delim(*d, done, self.stamp(*span)));
                    i += 1;
                }
                // A template cannot literally contain a fragment (it is
                // written source), but a compiled arm built by code can.
                Tt::Frag(kind, inner, span) => {
                    out.push(Tt::Frag(*kind, inner.clone(), *span));
                    i += 1;
                }
            }
        }
        Ok(out)
    }

    /// Handles `$name` (and `$crate`) at template position `i` (the `$`).
    /// Returns the next template index.
    fn metavar(&mut self, name_tok: Token, dollar_span: Span, i: usize, indices: &[usize], out: &mut Vec<Tt>) -> Result<usize, Diag> {
        if name_tok.tok == Tok::Crate {
            // `$crate` names the defining crate's root (ARCH 6.3). One
            // crate today: that is the `crate` path itself.
            out.push(Tt::Tok(Token { tok: Tok::Crate, span: self.stamp(dollar_span) }));
            return Ok(i + 2);
        }
        // An unbound name stays as the tokens `$` and the name in rustc's
        // transcriber; it is not an error. rustc writes a keyword name
        // back raw (`$if` becomes `$ r#if`), so it no longer matches the
        // keyword; a raw identifier here is a plain `Ident`.
        let unbound = |this: &mut Self, out: &mut Vec<Tt>| {
            let tok = match name_tok.tok {
                Tok::Reserved(s) => Tok::Ident(s),
                Tok::True | Tok::False => Tok::Ident(this.interner.intern(&tok_text(&name_tok.tok, &[], this.interner))),
                Tok::SelfLower | Tok::SelfUpper | Tok::Super | Tok::Crate | Tok::Ident(_) => name_tok.tok,
                t => meta_sym(t, this.interner).map_or(t, Tok::Ident),
            };
            out.push(Tt::Tok(Token { tok: Tok::Dollar, span: this.stamp(dollar_span) }));
            out.push(Tt::Tok(Token { tok, span: this.stamp(name_tok.span) }));
            Ok(i + 2)
        };
        let Some(name) = meta_sym(name_tok.tok, self.interner) else {
            if matches!(name_tok.tok, Tok::Reserved(_) | Tok::True | Tok::False) {
                return unbound(self, out);
            }
            // `$` before a token that cannot name a metavariable is a
            // literal `$`; the token itself is transcribed next round.
            out.push(Tt::Tok(Token { tok: Tok::Dollar, span: self.stamp(dollar_span) }));
            return Ok(i + 1);
        };
        let Some(bind) = self.binds.get(&name) else {
            return unbound(self, out);
        };
        match select(bind, indices) {
            Ok(Some(leaf)) => out.extend(leaf.iter().cloned()),
            Ok(None) => {}
            Err(SelectErr::StillRepeating) => {
                return Err(err(
                    C_DEF,
                    name_tok.span,
                    format!("metavariable `{}` is still repeating at this depth", self.interner.get(name)),
                    "Use it inside as many repetitions as bind it in the matcher.",
                ));
            }
        }
        Ok(i + 2)
    }

    /// Expands one template repetition into `out`.
    fn repetition(
        &mut self,
        inner: &[Tt],
        sep: Option<Token>,
        op: RepOp,
        span: Span,
        indices: &mut Vec<usize>,
        out: &mut Vec<Tt>,
    ) -> Result<(), Diag> {
        // The count comes from the variables this repetition drives:
        // after the indices outside it are applied, every variable used
        // inside that is still a sequence must agree on its length.
        let mut names = Vec::new();
        collect_vars(inner, self.interner, &mut names);
        let mut count: Option<usize> = None;
        for name in &names {
            let Some(bind) = self.binds.get(name) else { continue };
            let sub = walk(bind, indices);
            match sub {
                Some(Bind::Seq(items)) => match count {
                    None => count = Some(items.len()),
                    Some(c) if c != items.len() => {
                        return Err(err(
                            C_DEF,
                            span,
                            "metavariables in one repetition were matched different numbers of times",
                            "Match the repeated variables together, in one repetition.",
                        ));
                    }
                    _ => {}
                },
                // Bound by an optional group or iteration that did not
                // run: this repetition runs zero times.
                Some(Bind::Empty) => match count {
                    None => count = Some(0),
                    Some(c) if c != 0 => {
                        return Err(err(
                            C_DEF,
                            span,
                            "metavariables in one repetition were matched different numbers of times",
                            "Match the repeated variables together, in one repetition.",
                        ));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        let Some(count) = count else {
            return Err(err(
                C_DEF,
                span,
                "attempted to repeat an expression containing no syntax variables",
                "A repetition needs a metavariable matched inside a repetition of the same depth.",
            ));
        };
        if op == RepOp::Plus && count == 0 {
            return Err(err(
                C_DEF,
                span,
                "this repetition must repeat at least once",
                "Use `*` for a variable the matcher may leave empty.",
            ));
        }
        if op == RepOp::Question && count > 1 {
            return Err(err(C_DEF, span, "a `?` repetition matched more than once", "Use `*` for a variable that can repeat."));
        }
        for k in 0..count {
            if k > 0
                && let Some(sep) = sep
            {
                out.push(Tt::Tok(Token { tok: sep.tok, span: self.stamp(sep.span) }));
            }
            indices.push(k);
            let done = self.seq(inner, indices);
            indices.pop();
            out.extend(done?);
        }
        Ok(())
    }
}

enum SelectErr {
    StillRepeating,
}

/// Walks a binding down by repetition indices. Returns the sub-binding,
/// or None when an index runs past a leaf (a shallower variable reused
/// at a deeper level walks to itself, handled by `select`).
fn walk<'b>(bind: &'b Bind, indices: &[usize]) -> Option<&'b Bind> {
    let mut cur = bind;
    for &i in indices {
        match cur {
            Bind::Seq(items) => cur = items.get(i)?,
            _ => return Some(cur),
        }
    }
    Some(cur)
}

/// Picks one leaf's trees out of a binding at the current repetition
/// indices. `Ok(None)` is an `Empty` binding (nothing is emitted).
fn select<'b>(bind: &'b Bind, indices: &[usize]) -> Result<Option<&'b Vec<Tt>>, SelectErr> {
    match walk(bind, indices) {
        Some(Bind::Leaf(trees)) => Ok(Some(trees)),
        Some(Bind::Empty) | None => Ok(None),
        Some(Bind::Seq(_)) => Err(SelectErr::StillRepeating),
    }
}

/// Collects every metavariable name used in template trees, nested
/// groups and repetitions included.
fn collect_vars(trees: &[Tt], interner: &Interner, out: &mut Vec<Sym>) {
    let mut i = 0;
    while i < trees.len() {
        match &trees[i] {
            Tt::Tok(t) if t.tok == Tok::Dollar => match trees.get(i + 1) {
                Some(Tt::Tok(name_tok)) => {
                    if let Some(s) = meta_sym(name_tok.tok, interner)
                        && name_tok.tok != Tok::Crate
                        && !out.contains(&s)
                    {
                        out.push(s);
                    }
                    i += 2;
                }
                Some(Tt::Delim(Delim::Paren, inner, _)) => {
                    collect_vars(inner, interner, out);
                    i += 2;
                }
                _ => i += 1,
            },
            Tt::Delim(_, inner, _) => {
                collect_vars(inner, interner, out);
                i += 1;
            }
            _ => i += 1,
        }
    }
}

/// Checks a template at definition time: only the repetition shapes
/// rustc validates when the definition is read (a repetition needs its
/// operator, and `?` takes no separator). A lone `$`, an unbound name,
/// and other bad `$` shapes are not definition errors in rustc;
/// `transcribe` reports them if the arm ever expands.
pub fn check_template_vars(template: &[Tt], vars: &FxMap<Sym, VarInfo>, interner: &Interner) -> Result<(), Diag> {
    let _ = (vars, interner);
    fn go(trees: &[Tt]) -> Result<(), Diag> {
        let mut i = 0;
        while i < trees.len() {
            match &trees[i] {
                Tt::Tok(t) if t.tok == Tok::Dollar => {
                    let span = t.span;
                    match trees.get(i + 1) {
                        Some(Tt::Delim(Delim::Paren, inner, _)) => {
                            let (sep, op, used) = super::match_::rep_tail(trees, i + 2, true).ok_or_else(|| {
                                err(C_DEF, span, "repetition has no operator", "Write `$(...)*`, `$(...)+`, or `$(...)?`.")
                            })?;
                            if sep.is_some() && op == RepOp::Question {
                                return Err(err(
                                    C_DEF,
                                    span,
                                    "the `?` repetition operator does not take a separator",
                                    "Drop the separator, or use `*` for a separated repetition.",
                                ));
                            }
                            go(inner)?;
                            i += 2 + used;
                        }
                        // rustc reads what follows `$` when it compiles the
                        // macro, used arm or not: a name (any identifier
                        // or keyword, bound or not) or `crate` is fine, and
                        // so is a `$` that ends its group. `${..}` and `$$`
                        // are unstable metavariable expressions; anything
                        // else is an error.
                        Some(Tt::Tok(next)) if super::match_::is_rustc_ident(next.tok) => i += 2,
                        None => i += 1,
                        Some(next) => {
                            let what = match next {
                                Tt::Delim(Delim::Brace, _, _) => "`${..}` metavariable expressions are unstable",
                                Tt::Tok(t) if t.tok == Tok::Dollar => "`$$` metavariable expressions are unstable",
                                _ => "expected a metavariable name or `(` after `$`",
                            };
                            return Err(err(C_DEF, span, what, "Write `$name` or `$(...)*`; capture a `$` as a tt argument to forward it."));
                        }
                    }
                }
                Tt::Delim(_, inner, _) => {
                    go(inner)?;
                    i += 1;
                }
                _ => i += 1,
            }
        }
        Ok(())
    }
    go(template)
}
