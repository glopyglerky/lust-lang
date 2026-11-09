// Matcher compilation and token-tree matching (ARCH 6.2). An arm
// compiles to a tree of token, fragment, group, and repetition nodes
// (for the definition checks), then to flat locations matched the way
// rustc's macro parser matches (`rustc_expand::mbe::macro_parser`):
// every live matcher position advances in lockstep over the input, one
// token at a time. A metavariable is filled by one greedy fragment
// parse, run only when it is the one live position that can begin on
// the token there; two or more candidates where a fragment is one of
// them is the local-ambiguity error, more than one full match is the
// "multiple successful parses" error, and a fragment parse that fails
// fails the whole call (no later arm is tried), all as in rustc.
use super::*;
use crate::parser;

/// A repetition operator: `*`, `+`, or `?`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepOp {
    Star,
    Plus,
    Question,
}

/// One matcher node, for the definition checks (follow sets, empty
/// repetitions); `lay_out` turns the tree into the locations that
/// matching walks.
#[derive(Debug)]
pub enum MNode {
    Tok(Token),
    Meta { name: Sym, kind: FragKind, span: Span },
    Group(Delim, Vec<MNode>),
    Rep { inner: Vec<MNode>, sep: Option<Token>, op: RepOp, span: Span, vars: Vec<Sym> },
}

/// Every metavariable name bound in a node subtree, in order.
fn node_vars(nodes: &[MNode], out: &mut Vec<Sym>) {
    for node in nodes {
        match node {
            MNode::Meta { name, .. } => {
                if !out.contains(name) {
                    out.push(*name);
                }
            }
            MNode::Group(_, inner) | MNode::Rep { inner, .. } => node_vars(inner, out),
            MNode::Tok(_) => {}
        }
    }
}

struct Compile<'a> {
    interner: &'a Interner,
    vars: FxMap<Sym, VarInfo>,
}

impl<'a> Compile<'a> {
    /// Compiles one sequence of matcher trees at repetition depth `depth`.
    fn seq(&mut self, trees: &[Tt], depth: u32) -> Result<Vec<MNode>, Diag> {
        let mut out = Vec::new();
        let mut i = 0;
        while i < trees.len() {
            match &trees[i] {
                Tt::Tok(t) if t.tok == Tok::Dollar => {
                    let span = t.span;
                    match trees.get(i + 1) {
                        // A repetition: `$( inner ) sep? op`.
                        Some(Tt::Delim(Delim::Paren, inner, _)) => {
                            let inner = self.seq(inner, depth + 1)?;
                            let (sep, op, used) = rep_tail(trees, i + 2, false).ok_or_else(|| {
                                err(
                                    C_DEF,
                                    span,
                                    "repetition has no operator",
                                    "Write `$(...)*`, `$(...)+`, or `$(...)?`, with an optional separator token.",
                                )
                            })?;
                            if sep.is_some() && op == RepOp::Question {
                                return Err(err(
                                    C_DEF,
                                    span,
                                    "the `?` repetition operator does not take a separator",
                                    "Drop the separator, or use `*` for a separated repetition.",
                                ));
                            }
                            if depth >= 16 {
                                return Err(err(C_DEF, span, "repetitions nested too deep", "Split the macro into smaller macros."));
                            }
                            let mut vars = Vec::new();
                            node_vars(&inner, &mut vars);
                            out.push(MNode::Rep { inner, sep, op, span, vars });
                            i += 2 + used;
                        }
                        // A metavariable: `$name:kind`. `$crate` is
                        // transcription-only and never a matcher variable.
                        Some(Tt::Tok(name_tok)) => {
                            if name_tok.tok == Tok::Crate {
                                if tok_at(trees, i + 2).map(|t| t.tok) != Some(Tok::Colon) {
                                    // A bare `$crate` in a matcher is the
                                    // two literal tokens `$` and `crate`.
                                    out.push(MNode::Tok(*t));
                                    out.push(MNode::Tok(*name_tok));
                                    i += 2;
                                    continue;
                                }
                                return Err(err(
                                    C_DEF,
                                    span,
                                    "`$crate` cannot be used in a matcher",
                                    "`$crate` is only allowed in the expanded side of a rule.",
                                ));
                            }
                            let Some(name) = meta_sym(name_tok.tok, self.interner) else {
                                return Err(err(
                                    C_DEF,
                                    span,
                                    "expected a metavariable name after `$`",
                                    "Write `$name:kind`, for example `$e:expr`.",
                                ));
                            };
                            let colon = tok_at(trees, i + 2);
                            let kind_tok = tok_at(trees, i + 3);
                            if colon.map(|t| t.tok) != Some(Tok::Colon) || kind_tok.is_none() {
                                return Err(err(C_DEF, span, "expected `$name:kind`", "Write `$name:kind`, for example `$e:expr`."));
                            }
                            let kind_name = seg_text(kind_tok.unwrap().tok, self.interner).unwrap_or_default();
                            let kind = FragKind::from_name(&kind_name).ok_or_else(|| {
                                err(
                                    C_DEF,
                                    kind_tok.unwrap().span,
                                    format!("unknown fragment kind `{kind_name}`"),
                                    "Use expr, ident, ty, pat, stmt, block, item, path, meta, vis, literal, lifetime, or tt.",
                                )
                            })?;
                            let name_span = name_tok.span;
                            if self.vars.insert(name, VarInfo { kind, depth }).is_some() {
                                return Err(err(
                                    C_DEF,
                                    name_span,
                                    format!("metavariable `{}` is bound more than once", self.interner.get(name)),
                                    "Rename one of the metavariables.",
                                ));
                            }
                            out.push(MNode::Meta { name, kind, span: name_span });
                            i += 4;
                        }
                        _ => {
                            return Err(err(
                                C_DEF,
                                span,
                                "expected a metavariable or repetition after `$`",
                                "Write `$name:kind` or `$(...)*`.",
                            ));
                        }
                    }
                }
                Tt::Tok(t) => {
                    // A written `>>` or `>>=` is one matcher token.
                    if let Some(shr) = fused_shr(trees, i) {
                        let next = tok_at(trees, i + 1).unwrap();
                        out.push(MNode::Tok(Token { tok: shr, span: t.span.to(next.span) }));
                        i += 2;
                    } else {
                        out.push(MNode::Tok(*t));
                        i += 1;
                    }
                }
                Tt::Delim(d, inner, _) => {
                    let inner = self.seq(inner, depth)?;
                    out.push(MNode::Group(*d, inner));
                    i += 1;
                }
                Tt::Frag(_, _, span) => {
                    return Err(err(
                        C_DEF,
                        *span,
                        "a fragment cannot be used in a matcher",
                        "Macro definitions must be written as plain tokens.",
                    ));
                }
            }
        }
        Ok(out)
    }
}

/// The fused shift token at `trees[i]`, when a `>` there touches
/// a following `>` or `>=` (same file and context, spans back to
/// back): rustc's token trees hold `>>` and `>>=` whole, while the
/// lexer here emits the two halves so generics parse unsplit.
/// The matcher fuses them where rustc counts whole tokens (a
/// written `>>` in a matcher, a `tt` fragment, a separator); the
/// fragment parsers keep the halves, so an end inside a `>>`
/// (a path in `<$p:path>`) still splits exactly like rustc's.
fn fused_shr(trees: &[Tt], i: usize) -> Option<Tok> {
    let a = tok_at(trees, i)?;
    if a.tok != Tok::Gt {
        return None;
    }
    let b = tok_at(trees, i + 1)?;
    if a.span.file != b.span.file || a.span.ctxt != b.span.ctxt || b.span.lo != a.span.hi {
        return None;
    }
    match b.tok {
        Tok::Gt => Some(Tok::Shr),
        Tok::Ge => Some(Tok::ShrEq),
        _ => None,
    }
}

/// Parses what follows a repetition's closing paren at `trees[i..]`:
/// an optional single separator token, then `*`, `+`, or `?`.
/// Returns the separator, the operator, and how many trees were used.
/// Also used by the transcriber, which reads the same repetition shape.
pub fn rep_tail(trees: &[Tt], i: usize, allow_dollar_sep: bool) -> Option<(Option<Token>, RepOp, usize)> {
    fn as_op(tok: Tok) -> Option<RepOp> {
        Some(match tok {
            Tok::Star => RepOp::Star,
            Tok::Plus => RepOp::Plus,
            Tok::Question => RepOp::Question,
            _ => return None,
        })
    }
    // A `>>` or `>>=` separator is one token, like in a matcher.
    if let Some(shr) = fused_shr(trees, i) {
        if let (Some(a), Some(b), Some(op)) = (tok_at(trees, i), tok_at(trees, i + 1), tok_at(trees, i + 2).and_then(|t| as_op(t.tok))) {
            return Some((Some(Token { tok: shr, span: a.span.to(b.span) }), op, 3));
        }
    }
    match tok_at(trees, i) {
        Some(t) if as_op(t.tok).is_some() => Some((None, as_op(t.tok).unwrap(), 1)),
        Some(sep) => {
            // `$` cannot be a matcher separator (it can be one in a
            // transcriber), and the operator must follow.
            if sep.tok == Tok::Dollar && !allow_dollar_sep {
                return None;
            }
            let op = as_op(tok_at(trees, i + 1)?.tok)?;
            Some((Some(sep), op, 2))
        }
        None => None,
    }
}

/// What may follow a fragment of `kind` in a matcher: the follow set
/// rustc 1.99 (edition 2024) enforces at definition time (ARCH 6.2).
/// Probed against rustc per kind and token; only the restricted kinds
/// have a set at all, the rest accept any following token.
fn follow_ok(kind: FragKind, tok: Tok) -> bool {
    use Tok::*;
    match kind {
        FragKind::Expr | FragKind::Expr2021 | FragKind::Stmt => matches!(tok, FatArrow | Comma | Semi),
        // `>>` as a written token reached here fused; a single
        // `>` is also fine (it closes one angle level).
        FragKind::Ty | FragKind::Path => {
            matches!(tok, FatArrow | Comma | Semi | Eq | Pipe | Colon | Gt | Shr | LBracket | LBrace | As | Where)
        }
        // In edition 2024 `pat` still eats a top-level `|`, so `|` itself
        // may not follow it; `pat_param` stops before `|`, so it may.
        FragKind::Pat => matches!(tok, FatArrow | Comma | Eq | If | In),
        FragKind::PatParam => matches!(tok, FatArrow | Comma | Eq | Pipe | If | In),
        FragKind::Vis => vis_follow_ok(tok),
        _ => true,
    }
}

/// The `vis` follow set: a comma, or a token that can begin an item or a
/// type. Probed against rustc: every keyword and identifier qualifies,
/// as do `::`, `<`, `<<`, `&`, `&&`, `*`, `?`, `!`, `_`, lifetimes, and
/// paren/bracket groups; operators, literals, `:` and brace groups do not.
fn vis_follow_ok(tok: Tok) -> bool {
    use Tok::*;
    matches!(
        tok,
        Comma
            | Ident(_)
            | Reserved(_)
            | Lifetime(_)
            | Underscore
            | PathSep
            | Star
            | Amp
            | AndAnd
            | Shl
            | Lt
            | Question
            | Bang
            | LParen
            | LBracket
            | As
            | Break
            | Const
            | Continue
            | Else
            | Enum
            | False
            | Fn
            | For
            | If
            | Impl
            | In
            | Let
            | Loop
            | Match
            | Mod
            | Mut
            | Pub
            | Ref
            | Return
            | SelfLower
            | SelfUpper
            | Static
            | Struct
            | Trait
            | True
            | Type
            | Unsafe
            | Use
            | Where
            | While
            | Async
            | Dyn
            | Move
            | Crate
            | Super
            | Extern
            | Await
    )
}

/// Kinds whose follow set is restricted (see `follow_ok`).
fn restricted(kind: FragKind) -> bool {
    matches!(
        kind,
        FragKind::Expr
            | FragKind::Expr2021
            | FragKind::Stmt
            | FragKind::Ty
            | FragKind::Path
            | FragKind::Pat
            | FragKind::PatParam
            | FragKind::Vis
    )
}

/// Whether a fragment of kind `next` may start right after a fragment of
/// kind `prev`, per rustc: almost never, except a `block` (it starts
/// with `{`, which is in the ty/path follow set) and the item-shaped
/// starts a `vis` can precede.
fn nt_follow_ok(prev: FragKind, next: FragKind) -> bool {
    if !restricted(prev) {
        return true;
    }
    matches!(
        (prev, next),
        (FragKind::Ty, FragKind::Block)
            | (FragKind::Path, FragKind::Block)
            | (FragKind::Vis, FragKind::Ty)
            | (FragKind::Vis, FragKind::Path)
            | (FragKind::Vis, FragKind::Ident)
    )
}

/// True when one matcher node can match zero input trees.
fn nullable(node: &MNode) -> bool {
    match node {
        MNode::Tok(_) | MNode::Group(_, _) => false,
        MNode::Meta { kind, .. } => *kind == FragKind::Vis,
        MNode::Rep { inner, op, .. } => *op != RepOp::Plus || inner.iter().all(nullable),
    }
}

/// One thing that can come next after a fragment, for follow checks.
#[derive(Clone, Copy)]
enum First {
    Tok(Tok),
    Meta(FragKind),
}

/// Appends every `First` a node sequence can start with. Returns true
/// when the whole sequence can match empty, so what follows the
/// sequence is also a follow target.
fn first_of_seq(nodes: &[MNode], out: &mut Vec<First>) -> bool {
    for node in nodes {
        if !first_of_node(node, out) {
            return false;
        }
    }
    true
}

/// Appends every `First` one node can start with. Returns true when the
/// node can match empty.
fn first_of_node(node: &MNode, out: &mut Vec<First>) -> bool {
    match node {
        MNode::Tok(t) => {
            out.push(First::Tok(t.tok));
            false
        }
        MNode::Group(d, _) => {
            out.push(First::Tok(match d {
                Delim::Paren => Tok::LParen,
                Delim::Bracket => Tok::LBracket,
                Delim::Brace => Tok::LBrace,
            }));
            false
        }
        MNode::Meta { kind, .. } => {
            out.push(First::Meta(*kind));
            *kind == FragKind::Vis
        }
        MNode::Rep { inner, op, .. } => {
            let inner_empty = first_of_seq(inner, out);
            *op != RepOp::Plus || inner_empty
        }
    }
}

fn follow_err(kind: FragKind, span: Span, tok: Tok) -> Diag {
    let _ = tok;
    err(
        C_DEF,
        span,
        format!("`{}` fragments cannot be followed by this token", kind.name()),
        "Change the matcher so the fragment is followed by an allowed token (`=>`, `,`, or `;` always work).",
    )
    .with_label(span, format!("the `{}` fragment here", kind.name()))
}

/// Checks one fragment against every `First` that can follow it.
fn check_follow_firsts(kind: FragKind, span: Span, firsts: &[First]) -> Result<(), Diag> {
    for first in firsts {
        match first {
            First::Tok(tok) => {
                if !follow_ok(kind, *tok) {
                    return Err(follow_err(kind, span, *tok));
                }
            }
            First::Meta(next) => {
                if !nt_follow_ok(kind, *next) {
                    return Err(err(
                        C_DEF,
                        span,
                        format!("`{}` fragments cannot be followed by a `{}` fragment", kind.name(), next.name()),
                        "Put a token both sides can agree on between the fragments, such as `,` or `;`.",
                    ));
                }
            }
        }
    }
    Ok(())
}

/// The metavariables that can end a repetition body, with the nodes
/// after each inside the body already checked by the caller. Reaching
/// the end through a nullable tail also counts, so this walks the
/// sequence from the back while nodes are nullable.
fn end_metas(nodes: &[MNode]) -> Vec<(FragKind, Span)> {
    let mut out = Vec::new();
    for node in nodes.iter().rev() {
        match node {
            MNode::Meta { kind, span, .. } => out.push((*kind, *span)),
            MNode::Rep { inner, .. } => out.extend(end_metas(inner)),
            MNode::Group(_, _) | MNode::Tok(_) => {}
        }
        if !nullable(node) {
            break;
        }
    }
    out
}

/// Follow-set check over one matcher sequence, recursively. `after`
/// holds the `First`s that can follow this whole sequence (empty at an
/// arm end or a group end, where the fragment parser stops anyway).
fn check_follow_seq(nodes: &[MNode], after: &[First]) -> Result<(), Diag> {
    for (j, node) in nodes.iter().enumerate() {
        // What can follow node j: the rest of this sequence, then `after`.
        let mut next = Vec::new();
        if first_of_seq(&nodes[j + 1..], &mut next) {
            next.extend_from_slice(after);
        }
        match node {
            MNode::Meta { kind, span, .. } => check_follow_firsts(*kind, *span, &next)?,
            MNode::Group(_, inner) => check_follow_seq(inner, &[])?,
            MNode::Rep { inner, sep, .. } => {
                check_follow_seq(inner, &[])?;
                // A fragment at the end of the body is followed by the
                // separator between iterations and by whatever follows
                // the repetition after the last one. With no separator
                // rustc does not check the loop-back edge, so the body
                // start is not a follow target here.
                for (kind, span) in end_metas(inner) {
                    if let Some(sep) = sep {
                        check_follow_firsts(kind, span, &[First::Tok(sep.tok)])?;
                    }
                    check_follow_firsts(kind, span, &next)?;
                }
            }
            MNode::Tok(_) => {}
        }
    }
    Ok(())
}

/// rustc rejects a repetition whose body can match empty (it would not
/// make progress), unless a separator token stands between iterations.
/// Its test is shallow: every element a `vis` or a `*`/`?` repetition.
fn check_empty_reps(nodes: &[MNode]) -> Result<(), Diag> {
    for node in nodes {
        match node {
            MNode::Rep { inner, sep, span, .. } => {
                let empty = |node: &MNode| match node {
                    MNode::Meta { kind, .. } => *kind == FragKind::Vis,
                    MNode::Rep { op, .. } => *op != RepOp::Plus,
                    MNode::Tok(_) | MNode::Group(_, _) => false,
                };
                if sep.is_none() && inner.iter().all(empty) {
                    return Err(err(
                        C_DEF,
                        *span,
                        "repetition matches an empty token tree",
                        "Give the repetition a body that always consumes a token, or a separator.",
                    ));
                }
                check_empty_reps(inner)?;
            }
            MNode::Group(_, inner) => check_empty_reps(inner)?,
            MNode::Tok(_) | MNode::Meta { .. } => {}
        }
    }
    Ok(())
}

/// Compiles one macro arm: matcher trees to matcher nodes plus the
/// metavariable table. Runs the follow-set and empty-repetition
/// definition checks (ARCH 6.2), then lays the nodes out flat.
pub fn compile_arm(matcher: &[Tt], span: Span, interner: &Interner) -> Result<(ArmMatcher, FxMap<Sym, VarInfo>), Diag> {
    let mut c = Compile { interner, vars: FxMap::default() };
    let nodes = c.seq(matcher, 0)?;
    check_empty_reps(&nodes)?;
    check_follow_seq(&nodes, &[])?;
    let _ = span;
    let mut arm = ArmMatcher { locs: Vec::new(), rep_vars: Vec::new() };
    lay_out(&nodes, &mut arm);
    arm.locs.push(Loc::Eof);
    Ok((arm, c.vars))
}

/// One flat matcher location, laid out like rustc's `MatcherLoc`
/// (`compute_locs`): a matcher position is one index, and groups and
/// repetitions become jumps between indices.
#[derive(Debug)]
pub enum Loc {
    /// A literal token (a written `>>` is one fused token).
    Tok(Token),
    /// A group's opening or closing delimiter.
    Open(Delim),
    Close(Delim),
    /// A repetition start: enter the body at the next index, or (for
    /// `*` and `?`) skip to `after`. `rep` indexes `ArmMatcher::rep_vars`.
    Seq { op: RepOp, rep: u32, after: u32 },
    /// The end of a body with no separator: leave, or (not for `?`)
    /// start another iteration at `first`.
    EndNoSep { op: RepOp, first: u32 },
    /// The end of a body with a separator: leave (past the next index
    /// too), or take the separator.
    Sep(Token),
    /// Just past a separator: start another iteration at `first`.
    AfterSep { first: u32 },
    /// A metavariable, filled by one fragment parse.
    Meta { name: Sym, kind: FragKind },
    /// The end of the matcher.
    Eof,
}

/// One compiled arm: its flat locations, and per repetition the
/// metavariables bound inside it (each binds to a sequence, empty for
/// zero iterations).
#[derive(Debug)]
pub struct ArmMatcher {
    locs: Vec<Loc>,
    rep_vars: Vec<Vec<Sym>>,
}

/// Appends the flat locations of `nodes` to `arm` (rustc's `compute_locs`).
fn lay_out(nodes: &[MNode], arm: &mut ArmMatcher) {
    for node in nodes {
        match node {
            MNode::Tok(t) => arm.locs.push(Loc::Tok(*t)),
            MNode::Meta { name, kind, .. } => arm.locs.push(Loc::Meta { name: *name, kind: *kind }),
            MNode::Group(d, inner) => {
                arm.locs.push(Loc::Open(*d));
                lay_out(inner, arm);
                arm.locs.push(Loc::Close(*d));
            }
            MNode::Rep { inner, sep, op, vars, .. } => {
                let at = arm.locs.len();
                let rep = arm.rep_vars.len() as u32;
                arm.rep_vars.push(vars.clone());
                // Patched below, once the body's length is known.
                arm.locs.push(Loc::Eof);
                let first = (at + 1) as u32;
                lay_out(inner, arm);
                match sep {
                    Some(sep) => {
                        arm.locs.push(Loc::Sep(*sep));
                        arm.locs.push(Loc::AfterSep { first });
                    }
                    None => arm.locs.push(Loc::EndNoSep { op: *op, first }),
                }
                arm.locs[at] = Loc::Seq { op: *op, rep, after: arm.locs.len() as u32 };
            }
        }
    }
}

/// Match steps for one arm before the cap fires (ARCH 6.1, L0801). A
/// step is one matcher position looked at for one input token.
pub const MATCH_BUDGET: u32 = 2_000_000;

/// Per-call matcher state: who to ask for fragment probes, the step
/// budget, probe memoization, and the arenas behind matcher positions.
pub struct Matcher<'a> {
    pub interner: &'a Interner,
    pub known: &'a Known,
    pub strings: &'a [String],
    pub budget: u32,
    pub probe_cache: FxMap<(FragKind, u64), bool>,
    /// Set by a fragment sub-parser (a path's generic arguments, a
    /// literal's `-`) that began and failed during one fragment parse.
    parse_failed: bool,
    call_span: Span,
    /// Bindings of matcher positions (see `BindEntry`); handle 0 is
    /// the empty chain.
    binds: Vec<BindEntry>,
    /// Repetition iterations (see `IterEntry`).
    iters: Vec<IterEntry>,
    /// Open repetitions of matcher positions (see `Frame`).
    frames: Vec<Frame>,
}

/// Bindings, as entries in the matcher's arena: a handle is an index
/// into `Matcher::binds`, and each entry adds bindings on top of an
/// older handle. Extending a position's bindings pushes one entry, so
/// positions share their common history instead of copying it.
/// `match_arm` builds a `BindMap` only for the full match.
enum BindEntry {
    /// No bindings (handle 0).
    Root,
    /// `name` bound to fragment trees, on top of `prev`.
    Leaf { name: Sym, trees: Vec<Tt>, prev: u32 },
    /// The variables of repetition `rep` (an `ArmMatcher::rep_vars`
    /// index), each bound to a sequence over the iterations that end
    /// with `Matcher::iters[last]` (`NO_ITER`: zero), on top of `prev`.
    Rep { rep: u32, last: u32, prev: u32 },
}

/// One repetition iteration: the bindings of its own match and the
/// iteration before it (`NO_ITER` for the first).
struct IterEntry {
    binds: u32,
    prev: u32,
}

const NO_ITER: u32 = u32::MAX;

/// A repetition a matcher position is inside: the enclosing one
/// (`NO_FRAME`: none), the repetition, the bindings from before it,
/// and its last finished iteration (`NO_ITER`: none yet).
#[derive(Clone, Copy)]
struct Frame {
    outer: u32,
    rep: u32,
    before: u32,
    last: u32,
}

const NO_FRAME: u32 = u32::MAX;

/// One live matcher position (rustc's `MatcherPos`): a location, the
/// bindings so far in the innermost repetition iteration (or outside
/// all of them), and the innermost open repetition.
#[derive(Clone, Copy)]
struct Pos {
    loc: u32,
    binds: u32,
    frame: u32,
}

impl Pos {
    /// This position moved to `loc`, with the same bindings.
    fn at(self, loc: u32) -> Pos {
        Pos { loc, ..self }
    }
}

/// How one arm fared against a call (rustc's `ParseResult`).
pub enum ArmMatch {
    /// The arm matched, with these bindings.
    Match(BindMap),
    /// No match: the next arm gets its try.
    NoMatch,
    /// More than one parse was live where one had to be picked (local
    /// ambiguity at a fragment, or more than one full match): the call
    /// fails, as in rustc.
    Ambiguous,
    /// A fragment parse began and failed: the call fails without
    /// trying later arms, as in rustc.
    BadFragment,
}

impl<'a> Matcher<'a> {
    /// A fresh matcher for one macro call at `call_span`.
    pub fn new(interner: &'a Interner, known: &'a Known, strings: &'a [String], call_span: Span) -> Matcher<'a> {
        Matcher {
            interner,
            known,
            strings,
            budget: MATCH_BUDGET,
            probe_cache: FxMap::default(),
            parse_failed: false,
            call_span,
            binds: vec![BindEntry::Root],
            iters: Vec::new(),
            frames: Vec::new(),
        }
    }

    /// Adds one bindings entry and returns its handle.
    fn bind(&mut self, entry: BindEntry) -> u32 {
        self.binds.push(entry);
        (self.binds.len() - 1) as u32
    }

    /// Adds one repetition frame and returns its handle.
    fn frame(&mut self, frame: Frame) -> u32 {
        self.frames.push(frame);
        (self.frames.len() - 1) as u32
    }

    fn step(&mut self) -> Result<(), Diag> {
        self.budget = self.budget.saturating_sub(1);
        if self.budget == 0 {
            return Err(err(C_CAP, self.call_span, "macro matching exceeded its step limit", "Simplify the macro pattern or the call."));
        }
        Ok(())
    }

    /// Ends the current iteration of position `p`'s innermost
    /// repetition. Returns the position that leaves the repetition at
    /// `exit_loc` (its variables bound to all iterations so far), and
    /// the frame for one more iteration.
    fn end_iteration(&mut self, p: Pos, exit_loc: u32) -> (Pos, Frame) {
        let frame = self.frames[p.frame as usize];
        self.iters.push(IterEntry { binds: p.binds, prev: frame.last });
        let last = (self.iters.len() - 1) as u32;
        let binds = self.bind(BindEntry::Rep { rep: frame.rep, last, prev: frame.before });
        (Pos { loc: exit_loc, binds, frame: frame.outer }, Frame { last, ..frame })
    }
}

/// Expands transparent fragments in an input list into their trees, so a
/// forwarded ident, lifetime, or tt can be matched token by token.
/// Opaque fragments stay single units.
pub fn splice_transparent(input: &[Tt]) -> Vec<Tt> {
    let mut out = Vec::with_capacity(input.len());
    for tree in input {
        match tree {
            Tt::Frag(kind, inner, _) if kind.transparent() => out.extend(splice_transparent(inner)),
            _ => out.push(tree.clone()),
        }
    }
    out
}

/// The token under a `Cursor`.
#[derive(Clone, Copy)]
enum Cur {
    /// A token, how many trees it spans (2 for a glued `>>`/`>>=`), and
    /// whether it was written raw (`r#name`, see `raw_tok`).
    Tok(Tok, usize, bool),
    /// A group's opening delimiter.
    Open(Delim),
    /// The end of the group the cursor is in.
    Close(Delim),
    /// A forwarded opaque fragment, and whether it holds just one
    /// literal (an `expr` that can rebind as a `literal`).
    Frag(FragKind, bool),
    /// The end of the call input.
    Eof,
}

/// rustc's token cursor over the call input: one token at a time, a
/// group entered at its opening delimiter and left at its closing one.
/// Every live matcher position reads the same token.
struct Cursor {
    /// The open lists, outermost first: trees (transparent fragments
    /// spliced open), position, and the delimiter that closes the list
    /// (`None` for the call input itself).
    stack: Vec<(Vec<Tt>, usize, Option<Delim>)>,
}

impl Cursor {
    fn current(&self, m: &Matcher) -> Cur {
        let (trees, pos, delim) = self.stack.last().unwrap();
        match trees.get(*pos) {
            None => delim.map_or(Cur::Eof, Cur::Close),
            Some(Tt::Tok(t)) => match glued_at(trees, *pos) {
                Some(glued) => Cur::Tok(glued, 2, false),
                None => Cur::Tok(t.tok, 1, raw_tok(m, t)),
            },
            Some(Tt::Delim(d, _, _)) => Cur::Open(*d),
            Some(Tt::Frag(kind, inner, _)) => {
                Cur::Frag(*kind, matches!(kind, FragKind::Expr | FragKind::Expr2021) && trees_read_literal(inner))
            }
        }
    }

    /// Moves past the current token `cur` (entering or leaving a group).
    fn bump(&mut self, cur: Cur) {
        let (trees, pos, _) = self.stack.last_mut().unwrap();
        match cur {
            Cur::Tok(_, n, _) => *pos += n,
            Cur::Frag(..) => *pos += 1,
            Cur::Open(d) => {
                let Some(Tt::Delim(_, inner, _)) = trees.get(*pos) else { unreachable!("open delimiter without a group") };
                let inner = splice_transparent(inner);
                self.stack.push((inner, 0, Some(d)));
            }
            Cur::Close(_) => {
                self.stack.pop();
                self.stack.last_mut().unwrap().1 += 1;
            }
            Cur::Eof => {}
        }
    }
}

/// The glued token rustc's token trees hold at `trees[i]`: `>>` or
/// `>>=` when the `>` there starts a touching pair. rustc glues a run
/// of touching `>` left to right, so a `>` an odd number of places into
/// the run is the second half of a pair; the cursor stands there only
/// after a fragment parse split the pair (`Vec<Vec<u8>>` read as `ty`
/// up to the first `>`), and then it is a `>` of its own.
fn glued_at(trees: &[Tt], i: usize) -> Option<Tok> {
    let glued = fused_shr(trees, i)?;
    let mut start = i;
    while start > 0 && fused_shr(trees, start - 1) == Some(Tok::Shr) {
        start -= 1;
    }
    if (i - start) % 2 == 0 { Some(glued) } else { None }
}

/// True when matcher token `pattern` matches the current token: the same
/// token, and both raw or neither (rustc's `token_name_eq`).
fn cur_is(m: &Matcher, cur: Cur, pattern: &Token) -> bool {
    matches!(cur, Cur::Tok(actual, _, raw) if tok_matches(&pattern.tok, &actual, m.strings) && raw == raw_tok(m, pattern))
}

/// True when an identifier or lifetime token was written raw (`r#name`,
/// `'r#name`). The lexer keeps only the name; a raw token's span also
/// covers the `r#`.
fn raw_tok(m: &Matcher, t: &Token) -> bool {
    let len = t.span.hi.saturating_sub(t.span.lo) as usize;
    match t.tok {
        Tok::Ident(s) => len == m.interner.get(s).len() + 2,
        Tok::Lifetime(s) => len == m.interner.get(s).len() + 3,
        _ => false,
    }
}

/// Matches one arm against the call input, the way rustc's macro
/// parser does (see the file comment): all positions advance together,
/// and a fragment parse runs only when one metavariable alone can take
/// the current token.
pub fn match_arm(m: &mut Matcher, arm: &ArmMatcher, input: &[Tt]) -> Result<ArmMatch, Diag> {
    m.budget = MATCH_BUDGET;
    m.binds.truncate(1);
    m.iters.clear();
    m.frames.clear();
    let mut cursor = Cursor { stack: vec![(splice_transparent(input), 0, None)] };
    let mut cur = vec![Pos { loc: 0, binds: 0, frame: NO_FRAME }];
    let mut next: Vec<Pos> = Vec::new();
    loop {
        let token = cursor.current(m);
        next.clear();
        // Metavariables that can begin on this token, and positions at
        // the matcher's end (on the input's end only).
        let mut bb: Option<(Pos, Sym, FragKind)> = None;
        let mut bb_count = 0usize;
        let mut done: Option<Pos> = None;
        let mut done_count = 0usize;
        while let Some(p) = cur.pop() {
            m.step()?;
            match &arm.locs[p.loc as usize] {
                Loc::Tok(t) => {
                    if cur_is(m, token, t) {
                        next.push(p.at(p.loc + 1));
                    }
                }
                Loc::Open(d) => {
                    if matches!(token, Cur::Open(x) if x == *d) {
                        next.push(p.at(p.loc + 1));
                    }
                }
                Loc::Close(d) => {
                    if matches!(token, Cur::Close(x) if x == *d) {
                        next.push(p.at(p.loc + 1));
                    }
                }
                &Loc::Seq { op, rep, after } => {
                    if op != RepOp::Plus {
                        let binds = m.bind(BindEntry::Rep { rep, last: NO_ITER, prev: p.binds });
                        cur.push(Pos { loc: after, binds, frame: p.frame });
                    }
                    let frame = m.frame(Frame { outer: p.frame, rep, before: p.binds, last: NO_ITER });
                    cur.push(Pos { loc: p.loc + 1, binds: 0, frame });
                }
                &Loc::EndNoSep { op, first } => {
                    let (exit, again) = m.end_iteration(p, p.loc + 1);
                    cur.push(exit);
                    if op != RepOp::Question {
                        let frame = m.frame(again);
                        cur.push(Pos { loc: first, binds: 0, frame });
                    }
                }
                Loc::Sep(sep) => {
                    let (exit, again) = m.end_iteration(p, p.loc + 2);
                    cur.push(exit);
                    if cur_is(m, token, sep) {
                        let frame = m.frame(again);
                        next.push(Pos { loc: p.loc + 1, binds: 0, frame });
                    }
                }
                &Loc::AfterSep { first } => cur.push(p.at(first)),
                &Loc::Meta { name, kind } => {
                    if may_begin(m, kind, token) {
                        bb_count += 1;
                        bb = Some((p, name, kind));
                    }
                }
                Loc::Eof => {
                    if matches!(token, Cur::Eof) {
                        done_count += 1;
                        done = Some(p);
                    }
                }
            }
        }
        if matches!(token, Cur::Eof) {
            return Ok(match (done_count, done) {
                (1, Some(p)) => {
                    let mut map = BindMap::default();
                    materialize(m, &arm.rep_vars, p.binds, &mut map);
                    ArmMatch::Match(map)
                }
                (0, _) => ArmMatch::NoMatch,
                _ => ArmMatch::Ambiguous,
            });
        }
        if let Some((p, name, kind)) = bb {
            if bb_count > 1 || !next.is_empty() {
                return Ok(ArmMatch::Ambiguous);
            }
            let (trees, pos, _) = cursor.stack.last().unwrap();
            let Some((end, bound)) = bb_parse(m, kind, trees, *pos)? else { return Ok(ArmMatch::BadFragment) };
            cursor.stack.last_mut().unwrap().1 = end;
            let binds = m.bind(BindEntry::Leaf { name, trees: bound, prev: p.binds });
            cur.push(Pos { loc: p.loc + 1, binds, frame: p.frame });
            continue;
        }
        if next.is_empty() {
            return Ok(ArmMatch::NoMatch);
        }
        std::mem::swap(&mut cur, &mut next);
        cursor.bump(token);
    }
}

/// Builds the transcriber's bindings for one match from its arena chain
/// (see `BindEntry`). A newer entry wins over an older one.
fn materialize(m: &Matcher, rep_vars: &[Vec<Sym>], mut binds: u32, out: &mut BindMap) {
    loop {
        match &m.binds[binds as usize] {
            BindEntry::Root => return,
            BindEntry::Leaf { name, trees, prev } => {
                out.entry(*name).or_insert_with(|| Bind::Leaf(trees.clone()));
                binds = *prev;
            }
            BindEntry::Rep { rep, last, prev } => {
                // Each variable becomes one `Seq` deeper, with `Empty`
                // where an iteration did not bind it. Zero iterations
                // still bind every variable, to an empty `Seq`.
                let mut iters = Vec::new();
                let mut it = *last;
                while it != NO_ITER {
                    iters.push(m.iters[it as usize].binds);
                    it = m.iters[it as usize].prev;
                }
                let vars = &rep_vars[*rep as usize];
                let mut seqs: Vec<Vec<Bind>> = vars.iter().map(|_| Vec::with_capacity(iters.len())).collect();
                let mut map = BindMap::default();
                for &iter_binds in iters.iter().rev() {
                    map.clear();
                    materialize(m, rep_vars, iter_binds, &mut map);
                    for (seq, name) in seqs.iter_mut().zip(vars) {
                        seq.push(map.remove(name).unwrap_or(Bind::Empty));
                    }
                }
                for (seq, &name) in seqs.into_iter().zip(vars) {
                    out.entry(name).or_insert(Bind::Seq(seq));
                }
                binds = *prev;
            }
        }
    }
}

/// The span a fragment covers: first to last tree span in its slice.
fn frag_span(slice: &[Tt], at: Span) -> Span {
    match (slice.first(), slice.last()) {
        (Some(first), Some(last)) if first.span().file == last.span().file => first.span().to(last.span()),
        _ => at,
    }
}

/// True for tokens an `ident` fragment matches: an identifier or a
/// keyword (rustc accepts `self`, `fn`, `true`, `Self` here), never `_`.
fn is_ident_tok(tok: Tok) -> bool {
    matches!(
        tok,
        Tok::Ident(_)
            | Tok::Reserved(_)
            | Tok::As
            | Tok::Break
            | Tok::Const
            | Tok::Continue
            | Tok::Else
            | Tok::Enum
            | Tok::False
            | Tok::Fn
            | Tok::For
            | Tok::If
            | Tok::Impl
            | Tok::In
            | Tok::Let
            | Tok::Loop
            | Tok::Match
            | Tok::Mod
            | Tok::Mut
            | Tok::Pub
            | Tok::Ref
            | Tok::Return
            | Tok::SelfLower
            | Tok::SelfUpper
            | Tok::Static
            | Tok::Struct
            | Tok::Trait
            | Tok::True
            | Tok::Type
            | Tok::Unsafe
            | Tok::Use
            | Tok::Where
            | Tok::While
            | Tok::Async
            | Tok::Dyn
            | Tok::Move
            | Tok::Crate
            | Tok::Super
            | Tok::Extern
            | Tok::Await
    )
}

/// True for one literal token (a leading `-` is the caller's concern). A
/// doc comment's string counts: rustc's tokens hold `#[doc = r" x"]`.
fn is_literal_tok(tok: Tok) -> bool {
    matches!(
        tok,
        Tok::Int(..) | Tok::Float(_) | Tok::Str(..) | Tok::DocStr(_) | Tok::ByteStr(_) | Tok::ByteChar(_) | Tok::Char(..) | Tok::True | Tok::False
    )
}

/// True when these trees read as exactly one literal: one literal
/// token, or `-` before one, splicing nested fragments open. A
/// forwarded fragment that holds just a literal can rebind to a
/// `literal` metavariable whatever kind first matched it.
pub(super) fn trees_read_literal(trees: &[Tt]) -> bool {
    fn toks(trees: &[Tt], out: &mut Vec<Tok>) -> bool {
        trees.iter().all(|t| match t {
            Tt::Tok(tok) => {
                out.push(tok.tok);
                true
            }
            Tt::Frag(_, inner, _) => toks(inner, out),
            Tt::Delim(_, _, _) => false,
        })
    }
    let mut flat = Vec::new();
    if !toks(trees, &mut flat) {
        return false;
    }
    matches!(flat.as_slice(), [t] if is_literal_tok(*t)) || matches!(flat.as_slice(), [Tok::Minus, t] if is_literal_tok(*t))
}

/// True for tokens rustc lexes as identifiers: names, keywords
/// (reserved ones too), and `_`.
pub(super) fn is_rustc_ident(tok: Tok) -> bool {
    is_ident_tok(tok) || tok == Tok::Underscore
}

/// rustc's `ident_can_begin_expr`, for an identifier-like token: a
/// name, a path keyword, or one of the keywords that start expressions.
fn ident_can_begin_expr(m: &Matcher, tok: Tok) -> bool {
    use Tok::*;
    match tok {
        Ident(_) | SelfLower | SelfUpper | Super | Crate => true,
        Async | Break | Const | Continue | False | For | If | Let | Loop | Match | Move | Return | True | Unsafe | While | Static => true,
        Reserved(s) => matches!(m.interner.get(s), "do" | "box" | "gen" | "try" | "yield"),
        _ => false,
    }
}

/// rustc's `ident_can_begin_type`, for an identifier-like token.
fn ident_can_begin_type(m: &Matcher, tok: Tok) -> bool {
    use Tok::*;
    match tok {
        Ident(_) | SelfLower | SelfUpper | Super | Crate => true,
        Underscore | For | Impl | Fn | Unsafe | Extern | Dyn => true,
        Reserved(s) => m.interner.get(s) == "typeof",
        _ => false,
    }
}

/// rustc's `Token::can_begin_expr`.
fn can_begin_expr(m: &Matcher, cur: Cur) -> bool {
    use Tok::*;
    match cur {
        Cur::Tok(t, ..) if is_rustc_ident(t) => ident_can_begin_expr(m, t),
        Cur::Tok(t, ..) => {
            is_literal_tok(t)
                || matches!(
                    t,
                    Bang | Minus
                        | Star
                        | Pipe
                        | OrOr
                        | Amp
                        | AndAnd
                        | DotDot
                        | Ellipsis
                        | DotDotEq
                        | Lt
                        | Shl
                        | PathSep
                        | Lifetime(_)
                        | Pound
                )
        }
        Cur::Open(_) => true,
        Cur::Frag(kind, _) => matches!(kind, FragKind::Block | FragKind::Expr | FragKind::Expr2021 | FragKind::Literal | FragKind::Path),
        Cur::Close(_) | Cur::Eof => false,
    }
}

/// rustc's `Token::can_begin_type`.
fn can_begin_type(m: &Matcher, cur: Cur) -> bool {
    use Tok::*;
    match cur {
        Cur::Tok(t, ..) if is_rustc_ident(t) => ident_can_begin_type(m, t),
        Cur::Tok(t, ..) => matches!(t, Bang | Star | Amp | AndAnd | Question | Lifetime(_) | Lt | Shl | PathSep),
        Cur::Open(d) => d != Delim::Brace,
        Cur::Frag(kind, _) => matches!(kind, FragKind::Ty | FragKind::Path),
        Cur::Close(_) | Cur::Eof => false,
    }
}

/// rustc's `Token::can_begin_pattern`; `with_or` is the edition 2024
/// `pat` (a leading `|` begins it), not `pat_param`.
fn can_begin_pattern(cur: Cur, with_or: bool) -> bool {
    use Tok::*;
    match cur {
        Cur::Tok(t, ..) => {
            is_rustc_ident(t)
                || is_literal_tok(t)
                || matches!(t, Amp | Minus | AndAnd | DotDot | Ellipsis | PathSep | Lt | Shl)
                || (with_or && t == Pipe)
        }
        Cur::Open(d) => d != Delim::Brace,
        Cur::Frag(kind, _) => matches!(
            kind,
            FragKind::Expr
                | FragKind::Expr2021
                | FragKind::Literal
                | FragKind::Meta
                | FragKind::Pat
                | FragKind::PatParam
                | FragKind::Path
                | FragKind::Ty
        ),
        Cur::Close(_) | Cur::Eof => false,
    }
}

/// rustc's `nonterminal_may_begin_with`: true when a fragment of `kind`
/// may begin at the current token. A metavariable that cannot drops
/// out quietly; one that can is a fragment-parse candidate, and the
/// parse, once run, has no way back.
fn may_begin(m: &Matcher, kind: FragKind, cur: Cur) -> bool {
    let is = |tok: Tok| matches!(cur, Cur::Tok(t, ..) if t == tok);
    match kind {
        FragKind::Expr2021 => can_begin_expr(m, cur) && !is(Tok::Let) && !is(Tok::Const),
        FragKind::Expr => (can_begin_expr(m, cur) || is(Tok::Underscore)) && !is(Tok::Let),
        FragKind::Ty => can_begin_type(m, cur),
        FragKind::Ident => matches!(cur, Cur::Tok(t, ..) if is_ident_tok(t)),
        FragKind::Lifetime => matches!(cur, Cur::Tok(Tok::Lifetime(_), ..)),
        FragKind::Literal => match cur {
            Cur::Tok(t, ..) => is_literal_tok(t) || t == Tok::Minus,
            Cur::Frag(kind, reads_literal) => kind == FragKind::Literal || reads_literal,
            _ => false,
        },
        FragKind::Vis => match cur {
            Cur::Tok(t, ..) if t == Tok::Comma || is_rustc_ident(t) => true,
            Cur::Frag(..) => true,
            _ => can_begin_type(m, cur),
        },
        FragKind::Block => match cur {
            Cur::Open(d) => d == Delim::Brace,
            Cur::Frag(kind, _) => {
                matches!(kind, FragKind::Block | FragKind::Stmt | FragKind::Expr | FragKind::Expr2021 | FragKind::Literal)
            }
            _ => false,
        },
        FragKind::Path | FragKind::Meta => match cur {
            Cur::Tok(t, ..) => t == Tok::PathSep || is_rustc_ident(t),
            Cur::Frag(kind, _) => !matches!(kind, FragKind::Item | FragKind::Block | FragKind::Vis),
            _ => false,
        },
        FragKind::Pat => can_begin_pattern(cur, true),
        FragKind::PatParam => can_begin_pattern(cur, false),
        FragKind::Tt | FragKind::Item | FragKind::Stmt => !matches!(cur, Cur::Close(_) | Cur::Eof),
    }
}

/// rustc's fragment parse at `input[pos]` (`parse_nonterminal`), run once
/// a metavariable alone can begin there. Returns where the fragment ends
/// in `input` and the trees to bind, or None when the parse fails: rustc
/// reports that error, and the call fails without trying later arms.
fn bb_parse(m: &mut Matcher, kind: FragKind, input: &[Tt], pos: usize) -> Result<Option<(usize, Vec<Tt>)>, Diag> {
    // A forwarded opaque fragment (ARCH 6.2). rustc's parser takes one
    // of the kind it parses whole (`eat_metavar_seq`); expressions,
    // patterns, and statements also read a compatible fragment as
    // their first node and go on after it (`$e + 1` forwarded into an
    // `$a:expr` matcher is one expression). A `ty` stands in for a path
    // only when it is one (rustc's `MetaVarKind::Ty { is_path }`).
    if let Some(Tt::Frag(fk, inner, _)) = input.get(pos) {
        let fk = *fk;
        let whole = Some((pos + 1, vec![input[pos].clone()]));
        let path_ty = fk == FragKind::Ty && matches!(kind, FragKind::Path | FragKind::Meta) && ty_is_path(m, inner);
        let greedy = match kind {
            FragKind::Tt => return Ok(whole),
            FragKind::Expr | FragKind::Expr2021 | FragKind::Pat | FragKind::PatParam => can_forward(fk, kind),
            FragKind::Item | FragKind::Stmt if fk == FragKind::Vis => true,
            FragKind::Stmt if fk != kind => can_forward(fk, kind),
            FragKind::Ty if fk == FragKind::Path => true,
            FragKind::Path if path_ty => return Ok(whole),
            FragKind::Meta if fk == FragKind::Path || path_ty => true,
            FragKind::Literal if trees_read_literal(inner) => return Ok(whole),
            // Any other fragment is not `pub`: the visibility is empty.
            FragKind::Vis if fk != kind => return Ok(Some((pos, bound_trees(m, kind, input, pos, pos)))),
            _ if fk == kind => return Ok(whole),
            _ => return Ok(None),
        };
        if !greedy {
            return Ok(None);
        }
    }
    let Some(end) = fragment_end(m, kind, input, pos)? else { return Ok(None) };
    if parsed_kind(kind) || kind == FragKind::Meta {
        keyword_lifetimes(m, &input[pos..end], kind == FragKind::Meta)?;
    }
    Ok(Some((end, bound_trees(m, kind, input, pos, end))))
}

/// rustc's error for a lifetime or label named by a keyword (`'fn`; see
/// `lexer::keyword_lifetime_error`) in a fragment its parser just read.
/// rustc reports it even when the arm fails later, so the call fails.
/// Macro call and attribute arguments are token trees the fragment parse
/// does not read, and neither is a `meta`'s own argument group (`meta`
/// says so); forwarded fragments were checked when first bound.
fn keyword_lifetimes(m: &Matcher, trees: &[Tt], meta: bool) -> Result<(), Diag> {
    let tok = |i: usize| tok_at(trees, i).map(|t| t.tok);
    // `unsafe(..)` wraps the meta item itself.
    if let (true, Some(Tok::Unsafe), Some(Tt::Delim(_, inner, _))) = (meta, tok(0), trees.get(1)) {
        return keyword_lifetimes(m, inner, true);
    }
    let mut meta_args = meta;
    for (i, tree) in trees.iter().enumerate() {
        let prev = |k: usize| i.checked_sub(k).and_then(tok);
        match tree {
            Tt::Tok(t) => {
                meta_args &= t.tok != Tok::Eq;
                let Tok::Lifetime(name) = t.tok else { continue };
                // A label: after `break` or `continue`, or before `:` and a loop or block.
                let label = matches!(prev(1), Some(Tok::Break | Tok::Continue))
                    || (tok(i + 1) == Some(Tok::Colon)
                        && (matches!(tok(i + 2), Some(Tok::Loop | Tok::While | Tok::For))
                            || matches!(trees.get(i + 2), Some(Tt::Delim(Delim::Brace, _, _)))));
                let text = m.interner.get(name);
                if let Some(d) = crate::lexer::keyword_lifetime_error(text, raw_tok(m, t), label, t.span) {
                    return Err(d);
                }
            }
            Tt::Delim(_, inner, _) => {
                let call_args = prev(1) == Some(Tok::Bang) && prev(2).is_some_and(is_seg_tok);
                let rules_body = prev(2) == Some(Tok::Bang) && prev(1).is_some_and(is_seg_tok);
                let attr_body = prev(1) == Some(Tok::Pound) || (prev(1) == Some(Tok::Bang) && prev(2) == Some(Tok::Pound));
                if !(call_args || rules_body || attr_body || meta_args) {
                    keyword_lifetimes(m, inner, false)?;
                }
            }
            Tt::Frag(..) => {}
        }
    }
    Ok(())
}

/// True when a forwarded `ty` fragment's trees are one plain type path
/// (`a::B<C>`), which rustc lets stand in for a `path` (`Ty { is_path }`).
fn ty_is_path(m: &mut Matcher, inner: &[Tt]) -> bool {
    m.parse_failed = false;
    matches!(tok_at(inner, 0).map(|t| t.tok), Some(t) if is_seg_tok(t) || t == Tok::PathSep)
        && path_ends(m, inner, 0).last() == Some(&inner.len())
        && !m.parse_failed
}

/// The trees a fragment `input[pos..end]` binds: a whole forwarded
/// fragment as itself (keeping its kind identity), a transparent kind as
/// its raw trees, anything else wrapped as a new opaque fragment.
fn bound_trees(m: &Matcher, kind: FragKind, input: &[Tt], pos: usize, end: usize) -> Vec<Tt> {
    let slice = input[pos..end].to_vec();
    if let [Tt::Frag(..)] = slice.as_slice() {
        return slice;
    }
    if kind.transparent() {
        return slice;
    }
    let at = input.get(pos).map_or(m.call_span, |t| t.span());
    let span = frag_span(&slice, at);
    vec![Tt::Frag(kind, slice, span)]
}

/// The `;` a matched statement fragment needs when its trees go back to
/// the parser (`flatten_final`, `trees_from_tts`): rustc reads a `stmt`
/// metavariable as a whole statement, so a statement whose last tree is
/// not `;` and not a brace group gets one there. The fragment itself
/// keeps the tokens as matched (`stringify!` prints `let a = 1`).
pub fn stmt_semi(slice: &[Tt]) -> Option<Token> {
    match slice.last() {
        Some(Tt::Tok(t)) if t.tok == Tok::Semi => return None,
        Some(Tt::Delim(Delim::Brace, _, _)) => return None,
        Some(Tt::Frag(FragKind::Block, _, _)) => return None,
        _ => {}
    }
    // Items end where their own token shape ends (a trailing brace group
    // was handled above; a `;`-ended item already has its token).
    if let Some(Tt::Tok(t)) = slice.first()
        && matches!(t.tok, Tok::Fn | Tok::Struct | Tok::Enum | Tok::Impl | Tok::Mod | Tok::Trait)
    {
        return None;
    }
    let span = slice.last().map_or(Span::default(), |t| t.span());
    Some(Token { tok: Tok::Semi, span })
}

/// Whether an opaque fragment matched as `from` in an outer macro can
/// rebind to a `to` metavariable in an inner macro. Probed against
/// rustc per kind pair; anything not listed (and not the same kind, and
/// not `tt`, which takes every fragment whole) does not rebind.
fn can_forward(from: FragKind, to: FragKind) -> bool {
    use FragKind::*;
    // `expr_2021` and `expr` are one metavariable kind to rustc's parser.
    let from = if from == Expr2021 { Expr } else { from };
    let to = if to == Expr2021 { Expr } else { to };
    if from == to || to == Tt {
        return true;
    }
    matches!(
        (from, to),
        (Expr, Pat | PatParam | Stmt)
            | (Ty, Path | Meta)
            | (Path, Expr | Ty | Pat | PatParam | Stmt | Meta)
            | (Pat, PatParam)
            | (PatParam, Pat)
            | (Block, Expr | Stmt)
            | (Item, Stmt)
            | (Literal, Expr | Pat | PatParam | Stmt)
    )
}

/// Kinds rustc parses with its real fragment parser (as opposed to the
/// single-token scanner kinds): these read greedily and can go on past
/// their longest valid reading (see `continues`).
fn parsed_kind(kind: FragKind) -> bool {
    matches!(
        kind,
        FragKind::Expr
            | FragKind::Expr2021
            | FragKind::Ty
            | FragKind::Pat
            | FragKind::PatParam
            | FragKind::Path
            | FragKind::Stmt
            | FragKind::Block
            | FragKind::Item
    )
}

/// True when rustc's greedy parser, having read the longest valid
/// fragment `input[pos..end]`, would still go on with the tree at `end`
/// (and then fail, since no longer reading exists): a binary operator
/// after an expression, generic arguments after a type path, and so on.
fn continues(m: &Matcher, kind: FragKind, input: &[Tt], pos: usize, end: usize) -> bool {
    use Tok::*;
    let slice = &input[pos..end];
    let Some(next) = input.get(end) else { return false };
    let next_tok = match next {
        Tt::Tok(t) => Some(t.tok),
        _ => None,
    };
    let group = match next {
        Tt::Delim(d, _, _) => Some(*d),
        _ => None,
    };
    let head = tok_at(slice, 0).map(|t| t.tok);
    // A fragment that ends in a path (not a field after `.` or a cast's
    // type) goes on with what extends a path: `::`, and per kind a
    // macro call's `!`, call-style arguments, a struct body, generics.
    let before_last = slice.len().checked_sub(2).and_then(|i| tok_at(slice, i)).map(|t| t.tok);
    let path_end = matches!(slice.last(), Some(Tt::Tok(t)) if is_seg_tok(t.tok)) && !matches!(before_last, Some(Dot | As));
    let coupler = matches!(input.get(end + 1), Some(Tt::Delim(Delim::Brace, _, _))) || tok_at(input, end + 1).map(|t| t.tok) == Some(Star);
    let after_path = path_end
        && ((next_tok == Some(PathSep) && !coupler)
            || match kind {
                FragKind::Expr | FragKind::Expr2021 | FragKind::Stmt => next_tok == Some(Bang) || group == Some(Delim::Brace),
                FragKind::Ty => matches!(next_tok, Some(Bang | Lt | Shl)) || group == Some(Delim::Paren),
                FragKind::Path => matches!(next_tok, Some(Lt | Shl)) || group == Some(Delim::Paren),
                FragKind::Pat | FragKind::PatParam => next_tok == Some(Bang) || matches!(group, Some(Delim::Paren | Delim::Brace)),
                _ => false,
            });
    if after_path {
        return true;
    }
    match kind {
        FragKind::Expr | FragKind::Expr2021 | FragKind::Stmt => {
            // An item statement is complete. A statement that is a
            // block-like expression (`{}`, `if`, `match`, loops, `unsafe
            // {}`) is complete too, except for `.` and `?`.
            if kind == FragKind::Stmt {
                if super::starts_item(slice, m.interner) || matches!(slice, [Tt::Tok(t)] if t.tok == Semi) {
                    return false;
                }
                let block_like = (matches!(slice.last(), Some(Tt::Delim(Delim::Brace, _, _)))
                    && (matches!(slice.first(), Some(Tt::Delim(Delim::Brace, _, _)))
                        || matches!(head, Some(If | Match | Loop | While | For | Unsafe | Async | Const | Lifetime(_)))))
                    || matches!(slice, [Tt::Frag(FragKind::Block, _, _)]);
                if block_like {
                    return matches!(next_tok, Some(Dot | Question));
                }
            }
            matches!(group, Some(Delim::Paren | Delim::Bracket))
                || matches!(
                    next_tok,
                    Some(
                        Plus | Minus
                            | Star
                            | Slash
                            | Percent
                            | EqEq
                            | Ne
                            | Lt
                            | Le
                            | Gt
                            | Ge
                            | AndAnd
                            | OrOr
                            | Amp
                            | Pipe
                            | Caret
                            | Shl
                            | PlusEq
                            | MinusEq
                            | StarEq
                            | SlashEq
                            | PercentEq
                            | AmpEq
                            | PipeEq
                            | CaretEq
                            | ShlEq
                            | ShrEq
                            | Shr
                            | Eq
                            | Dot
                            | Question
                            | As
                            | DotDot
                            | DotDotEq
                            | Ellipsis
                    )
                )
                // `:`, `=`, and `else` after a leading `let` pattern.
                || (kind == FragKind::Stmt && head == Some(Let) && matches!(next_tok, Some(Colon | Eq | Else)))
        }
        // A fn type's return after its parenthesized argument list (a
        // `+` after a type is `ty_plus_end`'s).
        FragKind::Ty => next_tok == Some(Arrow) && matches!(slice.last(), Some(Tt::Delim(Delim::Paren, _, _))),
        FragKind::Pat | FragKind::PatParam => {
            let range_able = !matches!(slice.last(), Some(Tt::Delim(..)));
            // rustc reads `@` after any pattern (an error unless the
            // pattern was a binding that took it).
            (kind == FragKind::Pat && next_tok == Some(Pipe))
                || next_tok == Some(At)
                || (range_able && matches!(next_tok, Some(DotDot | DotDotEq | Ellipsis)))
        }
        _ => false,
    }
}

/// The end of a type `input[pos..end]` that a `+` follows, as rustc reads
/// it: after a path type, or a `dyn`/`impl` type, the `+` and the bounds
/// after it (possibly none) belong to the type; after any other type the
/// `+` is an error (E0178), None.
fn ty_plus_end(m: &mut Matcher, input: &[Tt], pos: usize, end: usize) -> Option<usize> {
    let head = tok_at(input, pos).map(|t| t.tok);
    let path_ty = matches!(head, Some(t) if is_seg_tok(t) || t == Tok::PathSep) && {
        m.parse_failed = false;
        let ok = path_ends(m, input, pos).last() == Some(&end) && !m.parse_failed;
        m.parse_failed = false;
        ok
    };
    if !path_ty && !matches!(head, Some(Tok::Dyn | Tok::Impl)) {
        return None;
    }
    super::recognize::bounds_tail_end(input, end, m.interner)
}

/// The end of rustc's greedy fragment of `kind` at `pos`, or None when
/// its parse fails: no reading at all, a sub-parse that began and
/// failed, or a longest reading the parser would go on past.
fn fragment_end(m: &mut Matcher, kind: FragKind, input: &[Tt], pos: usize) -> Result<Option<usize>, Diag> {
    m.parse_failed = false;
    let ends = fragment_ends(m, kind, input, pos)?;
    let Some(&end) = ends.iter().max() else { return Ok(None) };
    if m.parse_failed {
        return Ok(None);
    }
    let end = if kind == FragKind::Ty && input.get(end).is_some_and(|t| t.is_tok(Tok::Plus)) {
        match ty_plus_end(m, input, pos, end) {
            Some(end) => end,
            None => return Ok(None),
        }
    } else {
        end
    };
    if parsed_kind(kind) && continues(m, kind, input, pos, end) {
        return Ok(None);
    }
    Ok(Some(end))
}

/// The end positions where a fragment of `kind` starting at `pos` can
/// stop (`fragment_end` takes the longest: rustc's fragment parser is
/// greedy and never backtracks into a fragment to let the rest of the
/// matcher fit). Scanner kinds are exact. Parsed kinds try ends with
/// the real parser (ARCH 6.2). A sub-parse that began and failed sets
/// `Matcher::parse_failed`.
fn fragment_ends(m: &mut Matcher, kind: FragKind, input: &[Tt], pos: usize) -> Result<Vec<usize>, Diag> {
    let first = tok_at(input, pos).map(|t| t.tok);
    // A lifetime begins a type only as the first bound of a bare trait
    // object (`'a + Trait`); rustc rejects one with no `+` after it.
    if kind == FragKind::Ty && matches!(first, Some(Tok::Lifetime(_))) && tok_at(input, pos + 1).map(|t| t.tok) != Some(Tok::Plus) {
        return Ok(vec![]);
    }
    let mut ends = match kind {
        FragKind::Ident => match first {
            Some(t) if is_ident_tok(t) => vec![pos + 1],
            _ => vec![],
        },
        FragKind::Lifetime => match first {
            Some(Tok::Lifetime(_)) => vec![pos + 1],
            _ => vec![],
        },
        FragKind::Literal => match first {
            Some(t) if is_literal_tok(t) => vec![pos + 1],
            // A `-` before any literal token is one literal fragment in
            // rustc (even `- "x"`, which only fails later if typed).
            Some(Tok::Minus) if matches!(tok_at(input, pos + 1), Some(t) if is_literal_tok(t.tok)) => {
                vec![pos + 2]
            }
            // A `-` began the literal parse and no literal followed.
            _ => vec![],
        },
        FragKind::Tt => {
            if pos >= input.len() {
                vec![]
            } else if glued_at(input, pos).is_some() {
                // `>>` and `>>=` are one token tree in rustc.
                vec![pos + 2]
            } else {
                vec![pos + 1]
            }
        }
        FragKind::Block => match input.get(pos) {
            Some(tree @ Tt::Delim(Delim::Brace, _, _)) if probe(m, kind, std::slice::from_ref(tree)) => vec![pos + 1],
            _ => vec![],
        },
        FragKind::Vis => {
            // Greedy, like rustc: a `pub` here is the visibility, never
            // left for a later matcher token by matching empty instead.
            // The parenthesized form counts only for `pub(crate)`,
            // `pub(self)`, `pub(super)`, and `pub(in path)`; `pub(foo)`
            // is just `pub` (a tuple type may follow), but `pub(in`
            // commits to a module path and a closing paren.
            if first == Some(Tok::Pub) {
                if let Some(Tt::Delim(Delim::Paren, inner, _)) = input.get(pos + 1) {
                    if tok_at(inner, 0).map(|t| t.tok) == Some(Tok::In) {
                        if !mod_path(&inner[1..]) {
                            m.parse_failed = true;
                        }
                        vec![pos + 2]
                    } else if matches!(inner.as_slice(), [Tt::Tok(t)] if matches!(t.tok, Tok::Crate | Tok::SelfLower | Tok::Super)) {
                        vec![pos + 2]
                    } else {
                        vec![pos + 1]
                    }
                } else {
                    vec![pos + 1]
                }
            } else if pos < input.len() {
                vec![pos]
            } else {
                vec![]
            }
        }
        FragKind::Meta => meta_end(m, input, pos)?.into_iter().collect(),
        FragKind::Path => path_ends(m, input, pos),
        // An item ends at its item boundary (or after a macro call
        // item's group and `;`). A macro call shape is a valid item
        // token shape as written, so those ends stay unprobed; their
        // expansion is validated when it runs.
        FragKind::Item => {
            let mut ends = Vec::new();
            // The boundary alone is not validation: the slice must also
            // parse as exactly one item, or token soup with an item
            // keyword and a brace group would bind as an item fragment
            // rustc's item parser rejects.
            if let Some(end) = item_end(m, input, pos) {
                if probe(m, kind, &input[pos..end]) {
                    ends.push(end);
                }
            }
            if let Some(end) = macro_call_shape(input, pos) {
                // A macro call is one item only when brace-delimited
                // (complete at the brace) or followed by `;` (the item
                // macro rule rustc applies inside a fragment too).
                if matches!(input.get(end - 1), Some(Tt::Delim(Delim::Brace, _, _))) {
                    ends.push(end);
                } else if input.get(end).is_some_and(|t| t.is_tok(Tok::Semi)) {
                    ends.push(end + 1);
                }
            }
            // The probe fallback must not resurrect a macro call the
            // shape check just refused: this parser accepts a lone
            // paren macro call as a crate's last item, rustc does not.
            if ends.is_empty() && macro_call_shape(input, pos).is_none() {
                for end in pos + 1..=fragment_window(kind, input, pos, usize::MAX) {
                    m.step()?;
                    if probe(m, kind, &input[pos..end]) {
                        ends.push(end);
                    }
                }
            }
            ends.sort_unstable();
            ends.dedup();
            ends
        }
        FragKind::Expr | FragKind::Expr2021 => {
            // One greedy parse, the way rustc's fragment parser runs:
            // flatten the input once and ask the parser where one
            // expression ends, instead of probing every candidate end
            // (which is cubic on long repetitions). The flattened input
            // runs past the fragment's window by a few trees, not to the
            // end of the input (quadratic on long repetitions); see
            // `greedy_expr_end`.
            m.step()?;
            if kind == FragKind::Expr2021
                && (matches!(input.get(pos), Some(Tt::Tok(t)) if t.tok == Tok::Underscore)
                    && !matches!(input.get(pos + 1), Some(Tt::Tok(t)) if t.tok == Tok::Eq)
                    || tok_at(input, pos).map(|t| t.tok) == Some(Tok::Const)
                        && matches!(input.get(pos + 1), Some(Tt::Delim(Delim::Brace, _, _))))
            {
                vec![]
            } else if kind == FragKind::Expr && matches!(input.get(pos), Some(Tt::Tok(t)) if t.tok == Tok::Underscore) {
                vec![pos + 1]
            } else {
                match greedy_expr_end(m, kind, input, pos) {
                    Some(Some(end)) => vec![end],
                    Some(None) => vec![],
                    // The subset parser refuses some valid forms
                    // (closures, `?`, `.await`, async and unsafe
                    // blocks); the recognizer knows their shapes
                    // and where they greedily end.
                    None => {
                        super::recognize::expr_end(input, pos, m.interner, kind == FragKind::Expr).map_or_else(Vec::new, |end| vec![end])
                    }
                }
            }
        }
        // A lone `;` is an (empty) statement in rustc, and the whole one.
        FragKind::Stmt if first == Some(Tok::Semi) => vec![pos + 1],
        FragKind::Ty | FragKind::Pat | FragKind::PatParam | FragKind::Stmt => {
            let mut ends = Vec::new();
            let item_at = if kind == FragKind::Stmt { item_end(m, input, pos) } else { None };
            // Only the longest end counts (see `fragment_end`), so try ends
            // from the top of the fragment's window down and keep the first
            // that reads. Trying every end up to the end of the input made a
            // long repetition cubic.
            let top = fragment_window(kind, input, pos, usize::MAX).max(item_at.unwrap_or(0));
            for end in (pos + 1..=top).rev() {
                // A fragment ending in `..=` does not read (see below).
                if parsed_kind(kind) && input[end - 1].is_tok(Tok::DotDotEq) {
                    continue;
                }
                m.step()?;
                let slice = &input[pos..end];
                // A statement fragment never owns a `;`: a matcher that
                // wants the one after it writes the token itself. An item
                // statement is the exception: its own `;` belongs to the
                // item (`struct S;`).
                if kind == FragKind::Stmt && slice.iter().any(|t| t.is_tok(Tok::Semi)) && item_at != Some(end) {
                    continue;
                }
                // A pattern fragment cannot start with `..=`: rustc's
                // fragment parser does not begin a pattern there (a
                // subpattern after `@` may start with one, but that is
                // not this slice's head). The subset parser accepts it
                // as a range, so refuse it here.
                if matches!(kind, FragKind::Pat | FragKind::PatParam) && matches!(slice.first(), Some(Tt::Tok(t)) if t.tok == Tok::DotDotEq)
                {
                    continue;
                }
                if kind == FragKind::PatParam && slice.iter().any(|t| t.is_tok(Tok::Pipe)) {
                    continue;
                }
                if probe(m, kind, slice)
                    || (item_at == Some(end) && probe(m, FragKind::Item, slice))
                    || (kind == FragKind::Ty && is_fn_type(m, slice))
                {
                    ends.push(end);
                    break;
                }
            }
            ends
        }
    };
    // A fragment that ends in `..=` has no range end and does not
    // parse in rustc. The subset parser's range form accepts it,
    // so refuse those ends here for every parsed kind.
    if parsed_kind(kind) {
        ends.retain(|&end| !matches!(input.get(end.wrapping_sub(1)), Some(Tt::Tok(t)) if t.tok == Tok::DotDotEq));
    }
    Ok(ends)
}

/// The greedy expression parse at `input[pos]` (see `fragment_ends`):
/// Some(end in trees) when the parser reads one (Some(None) when it
/// ends inside a tree), None when the parser refuses the form. The
/// parser reads a flattened copy of a prefix of the trees, cut a little
/// past the fragment's window (`fragment_window`) at first. The parser
/// never backtracks and peeks at most `PARSE_LOOKAHEAD` tokens past
/// where it stops, so when it stops (or fails) that far before the cut,
/// the whole input reads the same; otherwise the cut doubles. A parse
/// of the whole rest per fragment made long repetitions quadratic.
fn greedy_expr_end(m: &Matcher, kind: FragKind, input: &[Tt], pos: usize) -> Option<Option<usize>> {
    let window = fragment_window(kind, input, pos, pos + 64);
    let mut cut = (window + 4).min(input.len());
    loop {
        let (flat, bounds) = probe_flatten_bounded(m, kind, &input[pos..cut]);
        let (flat, strings) = compact_strings(m, flat);
        let reach = parser::probe_expr_end(&flat, &strings, m.interner, m.known);
        let (Ok(read) | Err(read)) = reach;
        if cut < input.len() && read + PARSE_LOOKAHEAD >= flat.len() {
            cut = (pos + 2 * (cut - pos)).min(input.len());
            continue;
        }
        let end = reach.ok()?;
        return Some(match bounds.iter().position(|&b| b == end) {
            Some(k) if k > 0 => Some(pos + k),
            _ => None,
        });
    }
}

/// How many tokens past its position the parser may peek (it uses
/// `peek_at(3)` at most; the rest is slack).
const PARSE_LOOKAHEAD: usize = 8;

/// An upper bound on where a fragment of `kind` at `input[pos]` can end:
/// the index of the first tree no such fragment holds at its top level,
/// or `limit` (at most the input's length) when none comes first. A `;`
/// or `=>` ends every fragment (an item keeps its own `;`). A `,` or `=`
/// ends a type or pattern outside its `<..>`, as does a `>` that closes
/// no `<`; `if` and `in` end a pattern, and `|` a `pat_param`. In a
/// statement or expression a `,` ends the fragment outside a closure's
/// `|..|` parameters and a turbofish's `::<..>`, and only before any
/// other `<` (a comparison or a type's arguments, which this scan cannot
/// tell apart). The end search for a parsed fragment stops here, so a
/// long repetition costs time in proportion to its length.
fn fragment_window(kind: FragKind, input: &[Tt], pos: usize, limit: usize) -> usize {
    let expr_like = matches!(kind, FragKind::Expr | FragKind::Expr2021 | FragKind::Stmt | FragKind::Item);
    let pat = matches!(kind, FragKind::Pat | FragKind::PatParam);
    // `<..>` depth: every `<` in a type or pattern, a turbofish's in an expression.
    let mut depth = 0i64;
    let mut params_open = false;
    let mut comma_ends = true;
    let limit = limit.min(input.len());
    for j in pos..limit {
        let Tt::Tok(t) = &input[j] else { continue };
        let stop = match t.tok {
            Tok::Semi if kind == FragKind::Item => return j + 1,
            Tok::Semi | Tok::FatArrow => true,
            Tok::Comma => kind != FragKind::Item && depth == 0 && !(expr_like && (params_open || !comma_ends)),
            Tok::Eq => !expr_like && depth == 0,
            Tok::If | Tok::In => pat && depth == 0,
            Tok::Pipe => kind == FragKind::PatParam,
            _ => false,
        };
        if stop {
            return j;
        }
        let prev = if j > pos { input.get(j - 1) } else { None };
        match t.tok {
            Tok::Lt | Tok::Shl if expr_like && depth == 0 && !matches!(prev, Some(Tt::Tok(p)) if p.tok == Tok::PathSep) => comma_ends = false,
            Tok::Lt => depth += 1,
            Tok::Shl => depth += 2,
            Tok::Gt | Tok::Ge if depth > 0 || !expr_like => depth -= 1,
            Tok::Shr | Tok::ShrEq if depth > 0 || !expr_like => depth -= 2,
            // A `|` after an operand is `or`; anywhere else it opens a
            // closure's parameters, and the next `|` closes them.
            Tok::Pipe if expr_like && depth == 0 => params_open = !params_open && !prev.is_some_and(ends_operand),
            _ => {}
        }
        if depth < 0 && !expr_like {
            return j;
        }
    }
    limit
}

/// True when an expression can end with this tree, so a `|` after it is
/// the `or` operator. Anything not listed counts as no operand, which
/// reads the `|` as a closure's: the cautious choice for `fragment_window`.
fn ends_operand(tree: &Tt) -> bool {
    match tree {
        Tt::Delim(..) | Tt::Frag(FragKind::Expr | FragKind::Expr2021 | FragKind::Literal | FragKind::Path | FragKind::Block, _, _) => true,
        Tt::Frag(..) => false,
        Tt::Tok(t) => matches!(
            t.tok,
            Tok::Ident(_)
                | Tok::Int(..)
                | Tok::Float(_)
                | Tok::Str(..)
                | Tok::Char(..)
                | Tok::ByteStr(_)
                | Tok::ByteChar(_)
                | Tok::True
                | Tok::False
                | Tok::SelfLower
                | Tok::SelfUpper
                | Tok::Super
                | Tok::Crate
                | Tok::Question
                | Tok::Await
        ),
    }
}

/// The strings a probe's tokens name, as a table of their own, with the
/// tokens re-indexed into it. A probe parse copies its string table, and
/// the expander's grows with the file.
fn compact_strings(m: &Matcher, mut toks: Vec<Token>) -> (Vec<Token>, Vec<String>) {
    let mut strings = Vec::new();
    for t in &mut toks {
        for i in t.tok.string_indices_mut().into_iter().flatten() {
            strings.push(m.strings.get(*i as usize).cloned().unwrap_or_default());
            *i = strings.len() as u32 - 1;
        }
    }
    (toks, strings)
}

/// True when `trees` is exactly a module path, as `pub(in ...)` takes it.
pub(super) fn mod_path(trees: &[Tt]) -> bool {
    mod_path_end(trees, 0) == Some(trees.len())
}

/// rustc's end for an item at `pos` (see `super::item_end` for the
/// boundary). A `;` right after a closing brace belongs only to items
/// that end in `;` (`use a::{b};`, `const C: T = {..};`, `static`,
/// `type`); a braced `fn`, `struct`, `enum`, `impl`, `trait`, `mod`, or
/// macro call is complete at its brace, and the `;` is left over.
fn item_end(m: &Matcher, input: &[Tt], pos: usize) -> Option<usize> {
    if matches!(input.get(pos), Some(Tt::Frag(FragKind::Item, _, _))) {
        return Some(pos + 1);
    }
    let end = super::item_end(input, pos, m.interner)?;
    if end < pos + 2 || !input[end - 1].is_tok(Tok::Semi) || !matches!(input[end - 2], Tt::Delim(Delim::Brace, _, _)) {
        return Some(end);
    }
    // The item keyword, past attributes and a visibility.
    let mut i = pos;
    loop {
        match (tok_at(input, i).map(|t| t.tok), input.get(i + 1)) {
            (Some(Tok::Pound), Some(Tt::Delim(Delim::Bracket, _, _))) => i += 2,
            (Some(Tok::Pub), Some(Tt::Delim(Delim::Paren, _, _))) => i += 2,
            (Some(Tok::Pub), _) => i += 1,
            _ => break,
        }
    }
    let semi_item = match tok_at(input, i).map(|t| t.tok) {
        Some(Tok::Use | Tok::Static | Tok::Type) => true,
        // `const` items, not `const fn` (with `unsafe`/`async`/`extern`).
        Some(Tok::Const) => !input[i + 1..end].iter().take(4).any(|t| t.is_tok(Tok::Fn)),
        _ => false,
    };
    Some(if semi_item { end } else { end - 1 })
}

/// True when `slice` is one function pointer type (`fn(A) -> B`, with
/// optional `unsafe` and `extern "C"` prefixes). The parser has no fn
/// types yet, so `probe` cannot judge these; the shape is small enough
/// to validate directly, with each argument and the return type
/// probed as a type.
fn is_fn_type(m: &mut Matcher, slice: &[Tt]) -> bool {
    let mut i = 0;
    if tok_at(slice, i).map(|t| t.tok) == Some(Tok::Unsafe) {
        i += 1;
    }
    if tok_at(slice, i).map(|t| t.tok) == Some(Tok::Extern) {
        i += 1;
        if matches!(tok_at(slice, i).map(|t| t.tok), Some(Tok::Str(..))) {
            i += 1;
        }
    }
    if tok_at(slice, i).map(|t| t.tok) != Some(Tok::Fn) {
        return false;
    }
    let Some(Tt::Delim(Delim::Paren, args, _)) = slice.get(i + 1) else { return false };
    let args = args.clone();
    // Each argument piece reads as a type (a `name:` prefix is allowed
    // and not part of the type).
    let mut start = 0;
    for j in 0..=args.len() {
        if j < args.len() && !args[j].is_tok(Tok::Comma) {
            continue;
        }
        let piece = &args[start..j];
        start = j + 1;
        if piece.is_empty() {
            continue;
        }
        let ty_at = if piece.len() > 2 && tok_at(piece, 1).map(|t| t.tok) == Some(Tok::Colon) { 2 } else { 0 };
        if !probe(m, FragKind::Ty, &piece[ty_at..]) && !is_fn_type(m, &piece[ty_at..]) {
            return false;
        }
    }
    let rest = &slice[i + 2..];
    if rest.is_empty() {
        return true;
    }
    tok_at(rest, 0).map(|t| t.tok) == Some(Tok::Arrow)
        && rest.len() > 1
        && (probe(m, FragKind::Ty, &rest[1..]) || is_fn_type(m, &rest[1..]))
}

/// The end of one `meta` fragment at `pos` (rustc's `parse_attr_item`):
/// `unsafe(` an item `)`, or a module path and then its arguments: one
/// delimited group, `=` and a whole expression, or nothing. None when
/// rustc's parse fails.
fn meta_end(m: &mut Matcher, input: &[Tt], pos: usize) -> Result<Option<usize>, Diag> {
    if tok_at(input, pos).map(|t| t.tok) == Some(Tok::Unsafe) {
        let Some(Tt::Delim(Delim::Paren, inner, _)) = input.get(pos + 1) else { return Ok(None) };
        let inner = splice_transparent(inner);
        let end = meta_item_end(m, &inner, 0)?;
        return Ok((end == Some(inner.len())).then_some(pos + 2));
    }
    meta_item_end(m, input, pos)
}

/// A meta item's path and arguments at `pos` (see `meta_end`). The path
/// may be a forwarded `path` (or path-shaped `ty`) fragment.
fn meta_item_end(m: &mut Matcher, input: &[Tt], pos: usize) -> Result<Option<usize>, Diag> {
    let path_end = match input.get(pos) {
        Some(Tt::Frag(FragKind::Path | FragKind::Ty, inner, _)) => (!has_generic_args(inner)).then_some(pos + 1),
        _ => mod_path_end(input, pos),
    };
    let Some(end) = path_end else { return Ok(None) };
    match input.get(end) {
        Some(Tt::Delim(..)) => Ok(Some(end + 1)),
        Some(Tt::Tok(t)) if t.tok == Tok::Eq => {
            // A doc comment is `doc = DocStr`: the value is the doc
            // string itself, which no expression parse takes.
            if matches!(input.get(end + 1), Some(Tt::Tok(tv)) if matches!(tv.tok, Tok::DocStr(_))) {
                return Ok(Some(end + 2));
            }
            fragment_end(m, FragKind::Expr, input, end + 1)
        }
        _ => Ok(Some(end)),
    }
}

/// True when a forwarded path's trees hold generic arguments (`<..>` or
/// `(..) -> T` at the top level), which a module-style path may not have.
fn has_generic_args(trees: &[Tt]) -> bool {
    trees.iter().any(|t| match t {
        Tt::Tok(t) => matches!(t.tok, Tok::Lt | Tok::Shl),
        Tt::Frag(_, inner, _) => has_generic_args(inner),
        Tt::Delim(d, _, _) => *d == Delim::Paren,
    })
}

/// The end of a module-style path at `pos` (rustc's `PathStyle::Mod`):
/// an optional leading `::`, then segments joined by `::`, with no
/// generic arguments. A `::` needs a segment after it unless `{` or `*`
/// follows (where a use tree would go on). None when the parse fails.
fn mod_path_end(input: &[Tt], pos: usize) -> Option<usize> {
    let mut i = pos + usize::from(tok_at(input, pos).map(|t| t.tok) == Some(Tok::PathSep));
    loop {
        if !matches!(tok_at(input, i).map(|t| t.tok), Some(t) if is_seg_tok(t)) {
            return None;
        }
        i += 1;
        if tok_at(input, i).map(|t| t.tok) != Some(Tok::PathSep)
            || matches!(input.get(i + 1), Some(Tt::Delim(Delim::Brace, _, _)))
            || tok_at(input, i + 1).map(|t| t.tok) == Some(Tok::Star)
        {
            return Some(i);
        }
        i += 1;
    }
}

/// True when one comma piece of generic arguments has a reading as a
/// type, a const expression, a lifetime, or a `Name = ...` /
/// `Name: ...` binding. A piece with no reading (such as `1 2`) is
/// what rustc's path parser rejects; accepting it would bind a path
/// fragment rustc never accepts.
fn generic_arg_ok(m: &mut Matcher, piece: &[Tt]) -> bool {
    if piece.is_empty() {
        return false;
    }
    if matches!(piece, [Tt::Tok(t)] if matches!(t.tok, Tok::Lifetime(_))) {
        return true;
    }
    if probe(m, FragKind::Ty, piece) || probe(m, FragKind::Expr, piece) {
        return true;
    }
    // A binding: one name, then `=` or `:`, then a readable rest.
    if let (Some(name), Some(sep)) = (tok_at(piece, 0), tok_at(piece, 1))
        && super::meta_sym(name.tok, m.interner).is_some()
        && matches!(sep.tok, Tok::Eq | Tok::Colon)
        && piece.len() > 2
    {
        return generic_arg_ok(m, &piece[2..]);
    }
    false
}

/// True when the trees between a path's `<` and `>` read as generic
/// arguments (comma pieces, nested angles counted). Empty arguments
/// (`a<>`) are accepted by rustc's path parser, so they pass here too.
fn angle_args_ok(m: &mut Matcher, inner: &[Tt]) -> bool {
    if inner.is_empty() {
        return true;
    }
    let mut depth = 0i64;
    let mut start = 0;
    for i in 0..=inner.len() {
        let tok = tok_at(inner, i).map(|t| t.tok);
        match tok {
            Some(Tok::Lt) => depth += 1,
            Some(Tok::Shl) => depth += 2,
            Some(Tok::Shr) => depth -= 2,
            Some(Tok::Gt) => depth -= 1,
            _ => {}
        }
        if i == inner.len() || (tok == Some(Tok::Comma) && depth == 0) {
            if !generic_arg_ok(m, &inner[start..i]) {
                return false;
            }
            start = i + 1;
        }
    }
    true
}

/// Candidate ends for a path fragment: segments with optional generic
/// arguments. The caller keeps only the longest (greedy, like rustc).
/// An angle group whose arguments have no reading ends the path
/// before it instead of being balanced blind.
fn path_ends(m: &mut Matcher, input: &[Tt], pos: usize) -> Vec<usize> {
    fn angle_end(input: &[Tt], mut i: usize) -> Option<usize> {
        // `i` is at the opening `<`. Group trees are single units here;
        // their contents cannot hold this angle's closing token.
        let mut depth = 0i64;
        while i < input.len() {
            match tok_at(input, i).map(|t| t.tok) {
                Some(Tok::Lt) => depth += 1,
                Some(Tok::Shl) => depth += 2,
                Some(Tok::Shr) => {
                    depth -= 2;
                    if depth <= 0 {
                        return Some(i + 1);
                    }
                }
                Some(Tok::Gt) => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i + 1);
                    }
                }
                _ => {}
            }
            i += 1;
        }
        None
    }
    let mut ends = Vec::new();
    let mut i = pos;
    if tok_at(input, i).map(|t| t.tok) == Some(Tok::PathSep) {
        i += 1;
    }
    loop {
        if !matches!(tok_at(input, i).map(|t| t.tok), Some(t) if is_seg_tok(t)) {
            break;
        }
        i += 1;
        // Generic arguments: `::<..>` (turbofish) or `<..>`.
        let angle_at = if tok_at(input, i).map(|t| t.tok) == Some(Tok::PathSep) && tok_at(input, i + 1).map(|t| t.tok) == Some(Tok::Lt) {
            Some(i + 1)
        } else if tok_at(input, i).map(|t| t.tok) == Some(Tok::Lt) {
            Some(i)
        } else {
            None
        };
        if let Some(at) = angle_at {
            match angle_end(input, at) {
                Some(end) if angle_args_ok(m, &input[at + 1..end - 1]) => i = end,
                // The arguments started and failed (or never close):
                // rustc's path parse fails.
                _ => {
                    m.parse_failed = true;
                    break;
                }
            }
        }
        // Parenthesized arguments (`Fn(A, B)` sugar): the group is
        // part of the path when its pieces read as types, and a
        // following `-> T` return type belongs to it as well.
        if let Some(Tt::Delim(Delim::Paren, inner, _)) = input.get(i) {
            let inner = inner.clone();
            if angle_args_ok(m, &inner) {
                i += 1;
                if tok_at(input, i).map(|t| t.tok) == Some(Tok::Arrow) {
                    for end in (i + 2..=fragment_window(FragKind::Ty, input, i + 1, usize::MAX)).rev() {
                        if probe(m, FragKind::Ty, &input[i + 1..end]) || is_fn_type(m, &input[i + 1..end]) {
                            i = end;
                            break;
                        }
                    }
                }
            }
        }
        ends.push(i);
        if tok_at(input, i).map(|t| t.tok) == Some(Tok::PathSep) && matches!(tok_at(input, i + 1).map(|t| t.tok), Some(t) if is_seg_tok(t))
        {
            i += 1;
        } else {
            break;
        }
    }
    ends
}

/// Flattens a fragment candidate slice for a parser probe. Two things in
/// a slice are not plain syntax yet and become placeholders of the right
/// shape: opaque fragments forwarded from an outer macro, and macro
/// calls, which parse as one node of the position they stand in.
fn probe_flatten(m: &Matcher, kind: FragKind, trees: &[Tt]) -> Vec<Token> {
    probe_flatten_impl(m, kind, trees, None)
}

/// `probe_flatten` plus, per top-level tree, the flat index where it
/// starts (with one final entry for the end). A greedy parser end in
/// flat tokens maps back to a tree index through these bounds.
fn probe_flatten_bounded(m: &Matcher, kind: FragKind, trees: &[Tt]) -> (Vec<Token>, Vec<usize>) {
    let mut bounds = Vec::with_capacity(trees.len() + 1);
    let out = probe_flatten_impl(m, kind, trees, Some(&mut bounds));
    bounds.push(out.len());
    (out, bounds)
}

fn probe_flatten_impl(m: &Matcher, kind: FragKind, trees: &[Tt], bounds: Option<&mut Vec<usize>>) -> Vec<Token> {
    /// One node of `kind` where a macro call stands: the call can
    /// expand to anything that is valid there.
    fn placeholder(m: &Matcher, kind: FragKind, span: Span, out: &mut Vec<Token>) {
        let tok = |tok: Tok| Token { tok, span };
        match kind {
            FragKind::Ty | FragKind::Path => out.push(tok(Tok::Ident(m.interner.intern("u8")))),
            FragKind::Item => out.extend(
                [Tok::Fn, Tok::Ident(m.interner.intern("__lust_probe")), Tok::LParen, Tok::RParen, Tok::LBrace, Tok::RBrace].map(tok),
            ),
            FragKind::Pat | FragKind::PatParam => out.push(tok(Tok::Underscore)),
            FragKind::Block => out.extend([Tok::LBrace, Tok::RBrace].map(tok)),
            FragKind::Vis => {}
            _ => out.push(tok(Tok::Int(0, None, None))),
        }
    }
    /// A forwarded fragment of kind `frag` (followed by trees `rest`)
    /// inside a `ctx` probe: tokens that parse exactly where rustc
    /// accepts that kind of metavariable, as one unit that no neighbor
    /// merges into (rustc wraps it in invisible delimiters). A `ty` is
    /// `!` (a type, never an expression or pattern), a `pat` is `[ref p]`
    /// (never an expression, nothing after a path, no `@` after it), an
    /// `expr` is `0` (rustc's pattern parser takes one too), a `path` is
    /// a plain name, a `block` is no struct body after a path and no
    /// labeled block, a `stmt` is `let _ = 0;`, and kinds no position
    /// takes (a `meta` outside its own kind, an empty `vis` outside an
    /// item) are `$`, which no parse accepts.
    fn frag_placeholder(m: &Matcher, frag: FragKind, ctx: FragKind, rest: &[Tt], span: Span, out: &mut Vec<Token>) {
        let tok = |tok: Tok| Token { tok, span };
        let next = rest.first();
        let bad = tok(Tok::Dollar);
        match frag {
            FragKind::Ty => out.push(tok(Tok::Bang)),
            // A path fragment is a whole path: no `::` or struct body
            // extends it.
            FragKind::Path if next.is_some_and(|t| t.is_tok(Tok::PathSep) || matches!(t, Tt::Delim(Delim::Brace, _, _))) => out.push(bad),
            FragKind::Path => out.push(tok(Tok::Ident(m.interner.intern("u8")))),
            FragKind::Pat | FragKind::PatParam => {
                out.extend([Tok::LBracket, Tok::Ref, Tok::Ident(m.interner.intern("__lust_p")), Tok::RBracket].map(tok))
            }
            FragKind::Expr | FragKind::Expr2021 | FragKind::Literal => out.push(tok(Tok::Int(0, None, None))),
            // An inner block, so a path before it does not read it as
            // a struct literal body.
            FragKind::Block if matches!(out.as_slice(), [.., l, c] if matches!(l.tok, Tok::Lifetime(_)) && c.tok == Tok::Colon) => out.push(bad),
            FragKind::Block => out.extend([Tok::LBrace, Tok::LBrace, Tok::RBrace, Tok::RBrace].map(tok)),
            FragKind::Item => placeholder(m, frag, span, out),
            // A whole statement, valid only where one stands.
            FragKind::Stmt => out.extend([Tok::Let, Tok::Underscore, Tok::Eq, Tok::Int(0, None, None), Tok::Semi].map(tok)),
            FragKind::Vis if ctx == FragKind::Item || (ctx == FragKind::Stmt && super::starts_item(rest, m.interner)) => {}
            _ => out.push(bad),
        }
    }
    fn go(m: &Matcher, kind: FragKind, trees: &[Tt], out: &mut Vec<Token>, bounds: Option<&mut Vec<usize>>) {
        let mut bounds = bounds;
        let mut i = 0;
        while i < trees.len() {
            if let Some(b) = &bounds {
                debug_assert_eq!(b.len(), i);
            }
            if let Some(b) = &mut bounds {
                b.push(out.len());
            }
            // A macro call shape: segments, `!`, one group. Its argument
            // trees are arbitrary tokens to the fragment parser, so the
            // call reads as one placeholder node of this kind.
            if let Some(end) = macro_call_shape(trees, i) {
                let at = out.len();
                placeholder(m, kind, trees[i].span(), out);
                if let Some(b) = &mut bounds {
                    for _ in i + 1..end {
                        b.push(at);
                    }
                }
                i = end;
                continue;
            }
            // Lifetimes in types are the one rewritten token, as the
            // parser has no lifetime types yet: a generic argument (after
            // `<` or `,`) becomes a placeholder type, a reference's
            // lifetime (after `&`) and a trailing bound (`+ 'a`) are
            // dropped, and any other lifetime stays for the probe to
            // reject, as rustc's type parser does. Everything else
            // flattens as written: a placeholder that discards group
            // contents (an array, tuple, or index body) would let token
            // soup such as `[1 2]` or `(1 2, 3)` probe as a valid
            // fragment that rustc's fragment parser rejects, and a
            // template that only stringifies the fragment would then
            // accept the program.
            if let Tt::Tok(t) = &trees[i]
                && matches!(t.tok, Tok::Lifetime(_))
                && matches!(kind, FragKind::Ty | FragKind::Path)
            {
                match out.last().map(|x| x.tok) {
                    Some(Tok::Lt | Tok::Comma) => out.push(Token { tok: Tok::Ident(m.interner.intern("u8")), span: t.span }),
                    Some(Tok::Amp | Tok::AndAnd) => {}
                    Some(Tok::Plus) => {
                        out.pop();
                    }
                    _ => out.push(*t),
                }
                i += 1;
                continue;
            }
            match &trees[i] {
                Tt::Tok(t) => super::push_split_token(out, t),
                Tt::Delim(d, inner, span) => {
                    let (open, close) = match d {
                        Delim::Paren => (Tok::LParen, Tok::RParen),
                        Delim::Bracket => (Tok::LBracket, Tok::RBracket),
                        Delim::Brace => (Tok::LBrace, Tok::RBrace),
                    };
                    out.push(Token { tok: open, span: *span });
                    go(m, kind, inner, out, None);
                    out.push(Token { tok: close, span: *span });
                }
                Tt::Frag(fk, inner, span) => {
                    // Transparent kinds are their tokens; a block or item
                    // inside a probe of its own kind, and a `vis` with
                    // tokens, read as written.
                    if fk.transparent()
                        || (*fk == kind && matches!(kind, FragKind::Block | FragKind::Item))
                        || (*fk == FragKind::Vis && !inner.is_empty())
                    {
                        go(m, kind, inner, out, None);
                    } else {
                        frag_placeholder(m, *fk, kind, &trees[i + 1..], *span, out);
                    }
                }
            }
            i += 1;
        }
    }
    let mut out = Vec::new();
    let mut trees = trees;
    if kind == FragKind::Item {
        while tok_at(trees, 0).map(|t| t.tok) == Some(Tok::Pound) && matches!(trees.get(1), Some(Tt::Delim(Delim::Bracket, _, _))) {
            trees = &trees[2..];
        }
    }
    go(m, kind, trees, &mut out, bounds);
    out
}

/// If a macro call (path segments, `!`, delimited group) starts at
/// `trees[i]`, returns the index just past its group.
pub fn macro_call_shape(trees: &[Tt], i: usize) -> Option<usize> {
    let mut j = i;
    if tok_at(trees, j).map(|t| t.tok) == Some(Tok::PathSep) {
        j += 1;
    }
    loop {
        if !matches!(tok_at(trees, j).map(|t| t.tok), Some(t) if is_seg_tok(t)) {
            return None;
        }
        j += 1;
        match tok_at(trees, j).map(|t| t.tok) {
            Some(Tok::Bang) => {
                return match trees.get(j + 1) {
                    Some(Tt::Delim(_, _, _)) => Some(j + 2),
                    _ => None,
                };
            }
            Some(Tok::PathSep) => j += 1,
            _ => return None,
        }
    }
}

/// True when `slice` parses as exactly one fragment of `kind` with the
/// real parser. Memoized on the slice's token hash per call.
fn probe(m: &mut Matcher, kind: FragKind, slice: &[Tt]) -> bool {
    let key = (kind, hash_tts(slice, m.strings));
    if let Some(&hit) = m.probe_cache.get(&key) {
        return hit;
    }
    let (toks, strings) = compact_strings(m, probe_flatten(m, kind, slice));
    let ok = match kind {
        FragKind::Expr => parser::probe_expr(&toks, &strings, m.interner, m.known),
        FragKind::Ty => parser::probe_ty(&toks, &strings, m.interner, m.known),
        FragKind::Pat | FragKind::PatParam => parser::probe_pat(&toks, &strings, m.interner, m.known),
        FragKind::Block => parser::probe_block(&toks, &strings, m.interner, m.known),
        FragKind::Stmt => {
            parser::probe_stmt(&toks, &strings, m.interner, m.known) || {
                // A `let` statement is written without its `;` in a macro
                // call; try again with one, as a block would need it.
                let span = toks.last().map_or(Span::default(), |t| t.span);
                let mut with_semi = toks.clone();
                with_semi.push(Token { tok: Tok::Semi, span });
                parser::probe_stmt(&with_semi, &strings, m.interner, m.known)
            }
        }
        FragKind::Item => {
            let mut tokens = toks;
            let end = tokens.last().map_or(Span::default(), |t| t.span);
            tokens.push(Token { tok: Tok::Eof, span: end });
            let lexed = Lexed { tokens, strings };
            parser::parse(&lexed, m.interner, m.known).map_or(false, |ast| ast.items.len() == 1)
        }
        _ => false,
    };
    // The subset parser cannot yet parse every valid Rust form a
    // fragment can hold (closures, `?`, float patterns, a bare
    // `dyn`). rustc's fragment parser can, so a shape the parser
    // refuses gets one more chance from the token-tree recognizer,
    // which knows the full grammar but builds no AST.
    let ok = ok
        || match kind {
            FragKind::Expr => super::recognize::is_expr(slice, m.interner, true),
            FragKind::Expr2021 => super::recognize::is_expr(slice, m.interner, false),
            FragKind::Ty => super::recognize::is_ty(slice, m.interner),
            FragKind::Pat => super::recognize::is_pat(slice, m.interner, true),
            FragKind::PatParam => super::recognize::is_pat(slice, m.interner, false),
            FragKind::Stmt => super::recognize::is_stmt(slice, m.interner),
            FragKind::Item => super::recognize::is_item(slice, m.interner),
            FragKind::Block => matches!(slice, [one] if super::recognize::is_block(one, m.interner)),
            _ => false,
        };
    m.probe_cache.insert(key, ok);
    ok
}
