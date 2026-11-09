// The text rustc puts after "assertion failed: " for `assert!(cond)` with
// no message: `pprust::expr_to_string` of the parsed condition (rustc
// 1.99, rustc_ast_pretty). This file parses the unexpanded condition
// tokens with the grammar rustc uses and prints each node as pprust
// does, so spacing is normalized, trailing commas go, match arms get
// pprust's commas, and a `$e:expr` fragment gets the parentheses pprust
// adds for precedence. A nested macro call prints its tokens with
// rustc's token-stream printer (`print_tts`): a token written in this
// file keeps its source spacing, a token a `macro_rules!` body wrote is
// `Spacing::Alone`, and `space_between` drops the space in `f(`, `x,`,
// `a.b`, `#[`, `m!`.
//
// `cond_text` gives None for a form this printer does not model, and for
// a text rustc breaks over several lines (more than 78 bytes; lust has
// no line breaker): the caller rejects those with L0809, so lust never
// prints a text that differs from rustc's.
use super::*;
use crate::diag::CtxtId;
use crate::lexer::Tok;

/// rustc's pretty printer margin: a longer text breaks over lines.
const MARGIN: usize = 78;

#[derive(Clone, Copy, PartialEq)]
enum K {
    Tok(Tok),
    Open(Delim),
    Close(Delim),
    /// The invisible group rustc wraps around a `$x:expr` (and every other
    /// non-`tt`, non-`ident`, non-`lifetime`) fragment.
    FragOpen(FragKind),
    FragClose,
}

/// One token of the condition.
struct T {
    text: String,
    k: K,
    span: Span,
    /// rustc's `Spacing::Alone` after this token: a token stream prints a
    /// space here when `space_between` agrees. For a `Close` it is the
    /// group's close spacing, for an `Open` brace its open spacing. None
    /// when lust cannot know it (a macro body in another file).
    alone: Option<bool>,
    /// An identifier or lifetime a macro body substituted for one of its
    /// `$x`: rustc prints an `$x:ident` as an always-`Alone` NtIdent and
    /// an `$x:tt` with its source spacing, and lust's trees do not keep
    /// which one it was.
    subst: bool,
}

/// The pretty-printed condition of `assert!(trees)` (expander side), or
/// None when lust cannot print it as rustc does.
pub fn assert_cond_text(st: &Expander, trees: &[Tt]) -> Option<String> {
    let mut toks = Vec::new();
    flatten(st, trees, None, &mut toks);
    print(toks)
}

/// The pretty-printed condition from flat tokens of one file with no
/// macro_rules (each token's text, kind with delimiters as their own
/// tokens, and span), or None as for `assert_cond_text`. The parser calls
/// this for a file the expander skipped.
pub fn cond_text(flat: Vec<(String, Tok, Span)>) -> Option<String> {
    let spans: Vec<Span> = flat.iter().map(|f| f.2).collect();
    let toks = flat
        .into_iter()
        .enumerate()
        .map(|(i, (text, tok, span))| {
            // A gap before the next token is whitespace or a comment.
            let alone = spans.get(i + 1).is_none_or(|n| n.lo > span.hi || n.file != span.file);
            // The parser's text of a raw identifier has no `r#`.
            let raw = matches!(tok, Tok::Ident(_)) && (span.hi - span.lo) as usize == text.len() + 2;
            let text = if raw { format!("r#{text}") } else { text };
            T { text, k: kind_of(tok), span, alone: Some(alone), subst: false }
        })
        .collect();
    print(toks)
}

fn kind_of(tok: Tok) -> K {
    match tok {
        Tok::LParen => K::Open(Delim::Paren),
        Tok::LBracket => K::Open(Delim::Bracket),
        Tok::LBrace => K::Open(Delim::Brace),
        Tok::RParen => K::Close(Delim::Paren),
        Tok::RBracket => K::Close(Delim::Bracket),
        Tok::RBrace => K::Close(Delim::Brace),
        t => K::Tok(t),
    }
}

/// rustc's lexer: the token ending at `pos` is `Alone` when whitespace, a
/// comment, or the end of the file follows it.
fn src_alone(src: &str, pos: u32) -> bool {
    let rest = src.get(pos as usize..).unwrap_or("");
    match rest.chars().next() {
        None => true,
        Some(c) if is_pattern_ws(c) => true,
        Some('/') => rest.starts_with("//") || rest.starts_with("/*"),
        _ => false,
    }
}

/// Rust's Pattern_White_Space (what the lexer skips between tokens).
fn is_pattern_ws(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{85}' | '\u{200e}' | '\u{200f}' | '\u{2028}' | '\u{2029}')
}

/// The source spacing at `pos` after a delimiter of the group `span` that
/// a macro body wrote (rustc keeps a body's delimiter spacing), when that
/// body is in the file being expanded.
fn def_alone(st: &Expander, span: Span, pos: u32) -> Option<bool> {
    let src = st.cur_src.as_bytes();
    let here = st.root_file == Some(span.file) && st.cur_file == st.ctx.file_path;
    let delims = matches!(
        (src.get(span.lo as usize), span.hi.checked_sub(1).and_then(|h| src.get(h as usize))),
        (Some(b'(' | b'[' | b'{'), Some(b')' | b']' | b'}'))
    );
    (here && delims).then(|| src_alone(&st.cur_src, pos))
}

fn flatten(st: &Expander, trees: &[Tt], group: Option<CtxtId>, out: &mut Vec<T>) {
    for tree in trees {
        match tree {
            Tt::Tok(t) => {
                // A token this file wrote keeps its source spacing; one a
                // macro body wrote is `Alone` (rustc's transcriber).
                let alone = if t.span.ctxt == CtxtId::ROOT { src_alone(&st.cur_src, t.span.hi) } else { true };
                let subst = group.is_some_and(|c| c != t.span.ctxt) && (ident_like(t.tok) || matches!(t.tok, Tok::Lifetime(_)));
                // `>>` and `>>=` are one tree here; the parser sees `>` first.
                let second = match t.tok {
                    Tok::Shr => Some((">", Tok::Gt)),
                    Tok::ShrEq => Some((">=", Tok::Ge)),
                    _ => None,
                };
                if let Some((text, tok)) = second {
                    let lo = t.span.lo + 1;
                    out.push(T { text: ">".into(), k: K::Tok(Tok::Gt), span: Span { hi: lo, ..t.span }, alone: Some(false), subst: false });
                    out.push(T { text: text.into(), k: K::Tok(tok), span: Span { lo, ..t.span }, alone: Some(alone), subst: false });
                    continue;
                }
                out.push(T { text: builtin::stringify_tok_text(st, t), k: K::Tok(t.tok), span: t.span, alone: Some(alone), subst });
            }
            Tt::Delim(d, inner, span) => {
                let root = span.ctxt == CtxtId::ROOT;
                let at = |pos: u32| if root { Some(src_alone(&st.cur_src, pos)) } else { def_alone(st, *span, pos) };
                let (open, close) = match d {
                    Delim::Paren => ("(", ")"),
                    Delim::Bracket => ("[", "]"),
                    Delim::Brace => ("{", "}"),
                };
                let open_alone = if *d == Delim::Brace { at(span.lo + 1) } else { Some(false) };
                out.push(T { text: open.into(), k: K::Open(*d), span: Span { hi: span.lo + 1, ..*span }, alone: open_alone, subst: false });
                flatten(st, inner, Some(span.ctxt), out);
                // rustc's transcriber forces `Alone` after a body's `]`.
                let close_alone = if !root && *d == Delim::Bracket { Some(true) } else { at(span.hi) };
                let lo = span.hi.saturating_sub(1).max(span.lo + 1);
                out.push(T {
                    text: close.into(),
                    k: K::Close(*d),
                    span: Span { lo, hi: lo + 1, ..*span },
                    alone: close_alone,
                    subst: false,
                });
            }
            Tt::Frag(kind, inner, span) if !kind.transparent() => {
                out.push(T { text: String::new(), k: K::FragOpen(*kind), span: *span, alone: Some(false), subst: false });
                flatten(st, inner, None, out);
                out.push(T { text: String::new(), k: K::FragClose, span: *span, alone: Some(true), subst: false });
            }
            Tt::Frag(_, inner, _) => flatten(st, inner, group, out),
        }
    }
}

fn ident_like(t: Tok) -> bool {
    use Tok::*;
    matches!(
        t,
        Ident(_)
            | Reserved(_)
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
            | Underscore
    )
}

fn is_punct(t: Tok) -> bool {
    !ident_like(t)
        && !matches!(
            t,
            Tok::Int(..)
                | Tok::Float(_)
                | Tok::Str(..)
                | Tok::DocStr(_)
                | Tok::ByteStr(_)
                | Tok::ByteChar(_)
                | Tok::Char(..)
                | Tok::Lifetime(_)
        )
}

fn print(toks: Vec<T>) -> Option<String> {
    let mut p = P { t: toks, i: 0 };
    let e = p.expr(R::default())?;
    (p.i == p.t.len() && e.s.len() <= MARGIN).then_some(e.s)
}

/// rustc's `ExprPrecedence`, lowest first.
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
enum Prec {
    Jump,
    Assign,
    Range,
    LOr,
    LAnd,
    Compare,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Sum,
    Product,
    Cast,
    Prefix,
    Unambiguous,
}

impl Prec {
    fn next(self) -> Prec {
        use Prec::*;
        match self {
            Jump => Assign,
            Assign => Range,
            Range => LOr,
            LOr => LAnd,
            LAnd => Compare,
            Compare => BitOr,
            BitOr => BitXor,
            BitXor => BitAnd,
            BitAnd => Shift,
            Shift => Sum,
            Sum => Product,
            Product => Cast,
            Cast => Prefix,
            Prefix | Unambiguous => Unambiguous,
        }
    }
}

/// What the printing rules of an enclosing node look at.
#[derive(Clone, Copy, PartialEq)]
enum EK {
    Other,
    /// A plain or labeled block: a match arm body with no comma.
    Block,
    /// `if`, `match`, loops, `unsafe` blocks: `classify::expr_is_complete`.
    BlockLike,
    Cast,
    Let,
    /// A named field (`(s.f)()` needs parens as a callee).
    Field,
    /// A float literal ending in `.`, which takes a space before `.x`.
    DotLit,
    /// `..x` or `..`.
    PrefixRange,
}

struct Ex {
    s: String,
    prec: Prec,
    kind: EK,
    /// A `$x:expr` fragment: its parentheses depend on pprust's precedence
    /// fixups, which this printer follows only for plain operands.
    frag: bool,
}

impl Ex {
    fn new(s: String, prec: Prec, kind: EK) -> Ex {
        Ex { s, prec, kind, frag: false }
    }
    fn block_like(&self) -> bool {
        matches!(self.kind, EK::Block | EK::BlockLike)
    }
}

/// Parser restrictions: no struct literal (an `if` condition), and
/// statement position (a block-like expression ends the statement).
#[derive(Clone, Copy, Default)]
struct R {
    no_struct: bool,
    stmt: bool,
}

/// The precedence pprust's fixup gives `e` as a rightmost operand: a
/// closure, a jump with a value, or `..x` extends to the end there, so
/// it counts as a prefix expression.
fn eff(e: &Ex) -> Prec {
    if e.prec == Prec::Jump || e.kind == EK::PrefixRange { Prec::Prefix } else { e.prec }
}

/// A condition (`if`, `while`, `match`, `for`): pprust parenthesizes a
/// jump or closure there, which the source cannot have written bare.
fn cond(e: Ex) -> Option<String> {
    (e.prec != Prec::Jump).then_some(e.s)
}

/// `inner`, in parentheses when `paren` holds. A fragment that pprust
/// would print through a statement or jump fixup is not modeled.
fn wrap(e: Ex, paren: bool) -> Option<String> {
    if e.frag && (matches!(e.prec, Prec::Jump) || e.kind == EK::PrefixRange) {
        return None;
    }
    Some(if paren { format!("({})", e.s) } else { e.s })
}

#[derive(Clone, Copy)]
enum Op {
    Bin(&'static str, Prec),
    Assign(&'static str),
    Range(&'static str),
    Cast,
}

struct P {
    t: Vec<T>,
    i: usize,
}

impl P {
    fn kind(&self, i: usize) -> Option<K> {
        self.t.get(i).map(|t| t.k)
    }
    fn peek(&self) -> Option<K> {
        self.kind(self.i)
    }
    fn tok(&self) -> Option<Tok> {
        match self.peek() {
            Some(K::Tok(t)) => Some(t),
            _ => None,
        }
    }
    fn tok_at(&self, i: usize) -> Option<Tok> {
        match self.kind(i) {
            Some(K::Tok(t)) => Some(t),
            _ => None,
        }
    }
    fn is(&self, t: Tok) -> bool {
        self.tok() == Some(t)
    }
    fn eat(&mut self, t: Tok) -> bool {
        let hit = self.is(t);
        if hit {
            self.i += 1;
        }
        hit
    }
    fn expect(&mut self, t: Tok) -> Option<()> {
        self.eat(t).then_some(())
    }
    fn open(&self, d: Delim) -> bool {
        self.peek() == Some(K::Open(d))
    }
    fn close(&self, d: Delim) -> bool {
        self.peek() == Some(K::Close(d))
    }
    fn expect_close(&mut self, d: Delim) -> Option<()> {
        self.close(d).then(|| self.i += 1)
    }
    /// The current token's text, consumed.
    fn bump(&mut self) -> String {
        self.i += 1;
        self.t[self.i - 1].text.clone()
    }
    /// True when the token at `i` touches the one after it (`>` `>`).
    fn joint(&self, i: usize) -> bool {
        match (self.t.get(i), self.t.get(i + 1)) {
            (Some(a), Some(b)) => a.span.hi == b.span.lo && a.span.file == b.span.file && a.span.ctxt == b.span.ctxt,
            _ => false,
        }
    }

    /// `a, b, c` up to the close of `d` (consumed), trailing comma allowed.
    /// Returns the items and whether a trailing comma was written.
    fn list<X>(&mut self, d: Delim, mut item: impl FnMut(&mut P) -> Option<X>) -> Option<(Vec<X>, bool)> {
        let mut out = Vec::new();
        let mut trailing = false;
        while !self.close(d) {
            out.push(item(self)?);
            trailing = self.eat(Tok::Comma);
            if !trailing && !self.close(d) {
                return None;
            }
        }
        self.i += 1;
        Some((out, trailing))
    }

    // ---------- expressions ----------

    fn expr(&mut self, r: R) -> Option<Ex> {
        self.assoc(Prec::Jump, r)
    }

    /// The binary operator at the cursor and its token count.
    fn binop(&self) -> Option<(Op, usize)> {
        use Tok::*;
        let op = match self.tok()? {
            Plus => Op::Bin("+", Prec::Sum),
            Minus => Op::Bin("-", Prec::Sum),
            Star => Op::Bin("*", Prec::Product),
            Slash => Op::Bin("/", Prec::Product),
            Percent => Op::Bin("%", Prec::Product),
            Caret => Op::Bin("^", Prec::BitXor),
            Amp => Op::Bin("&", Prec::BitAnd),
            Pipe => Op::Bin("|", Prec::BitOr),
            AndAnd => Op::Bin("&&", Prec::LAnd),
            OrOr => Op::Bin("||", Prec::LOr),
            Shl => Op::Bin("<<", Prec::Shift),
            EqEq => Op::Bin("==", Prec::Compare),
            Ne => Op::Bin("!=", Prec::Compare),
            Lt => Op::Bin("<", Prec::Compare),
            Le => Op::Bin("<=", Prec::Compare),
            Ge => Op::Bin(">=", Prec::Compare),
            Gt if self.joint(self.i) && self.tok_at(self.i + 1) == Some(Gt) => return Some((Op::Bin(">>", Prec::Shift), 2)),
            Gt if self.joint(self.i) && self.tok_at(self.i + 1) == Some(Ge) => return Some((Op::Assign(">>="), 2)),
            Gt => Op::Bin(">", Prec::Compare),
            Eq => Op::Assign("="),
            PlusEq => Op::Assign("+="),
            MinusEq => Op::Assign("-="),
            StarEq => Op::Assign("*="),
            SlashEq => Op::Assign("/="),
            PercentEq => Op::Assign("%="),
            CaretEq => Op::Assign("^="),
            AmpEq => Op::Assign("&="),
            PipeEq => Op::Assign("|="),
            ShlEq => Op::Assign("<<="),
            DotDot => Op::Range(".."),
            DotDotEq => Op::Range("..="),
            As => Op::Cast,
            _ => return None,
        };
        Some((op, 1))
    }

    /// rustc's `parse_expr_assoc_with`: operators of precedence `min` and up.
    fn assoc(&mut self, min: Prec, r: R) -> Option<Ex> {
        let mut lhs = if self.is(Tok::DotDot) || self.is(Tok::DotDotEq) { self.prefix_range(r)? } else { self.unary(r)? };
        loop {
            if r.stmt && lhs.block_like() {
                return Some(lhs);
            }
            let Some((op, n)) = self.binop() else { return Some(lhs) };
            let prec = match op {
                Op::Bin(_, p) => p,
                Op::Assign(_) => Prec::Assign,
                Op::Range(_) => Prec::Range,
                Op::Cast => Prec::Cast,
            };
            if prec < min {
                return Some(lhs);
            }
            self.i += n;
            let r2 = R { stmt: false, ..r };
            lhs = match op {
                Op::Cast => {
                    let ty = self.ty()?;
                    let lp = lhs.prec < Prec::Cast;
                    let l = wrap(lhs, lp)?;
                    Ex::new(format!("{l} as {ty}"), Prec::Cast, EK::Cast)
                }
                Op::Assign(o) => {
                    let rhs = self.assoc(Prec::Assign, r2)?;
                    let lp = lhs.prec <= Prec::Range;
                    let rp = eff(&rhs) < Prec::Assign;
                    Ex::new(format!("{} {o} {}", wrap(lhs, lp)?, wrap(rhs, rp)?), Prec::Assign, EK::Other)
                }
                Op::Range(o) => {
                    if lhs.prec < Prec::LOr {
                        return None;
                    }
                    let dot = if lhs.kind == EK::DotLit { " " } else { "" };
                    let l = wrap(lhs, false)?;
                    let rhs = if self.range_rhs(r2) { Some(self.assoc(Prec::LOr, r2)?) } else { None };
                    let rs = match rhs {
                        Some(e) => {
                            let p = eff(&e) < Prec::LOr;
                            wrap(e, p)?
                        }
                        None => String::new(),
                    };
                    Ex::new(format!("{l}{dot}{o}{rs}"), Prec::Range, EK::Other)
                }
                Op::Bin(o, p) => {
                    let comparison = p == Prec::Compare;
                    let rhs = self.assoc(p.next(), r2)?;
                    let mut lp = if comparison { lhs.prec <= p } else { lhs.prec < p };
                    let rp = eff(&rhs) <= p;
                    if lhs.kind == EK::Cast && (o == "<" || o == "<<") || lhs.kind == EK::Let && p > Prec::LAnd {
                        lp = true;
                    }
                    Ex::new(format!("{} {o} {}", wrap(lhs, lp)?, wrap(rhs, rp)?), p, EK::Other)
                }
            };
        }
    }

    /// rustc's `is_at_start_of_range_notation_rhs`.
    fn range_rhs(&self, r: R) -> bool {
        if self.open(Delim::Brace) {
            return !r.no_struct;
        }
        self.can_begin_expr()
    }

    fn can_begin_expr(&self) -> bool {
        use Tok::*;
        match self.peek() {
            Some(K::Open(_)) => true,
            Some(K::FragOpen(k)) => matches!(k, FragKind::Expr | FragKind::Expr2021 | FragKind::Literal | FragKind::Block | FragKind::Path),
            Some(K::Tok(t)) => matches!(
                t,
                Ident(_)
                    | Int(..)
                    | Float(_)
                    | Str(..)
                    | ByteStr(_)
                    | ByteChar(_)
                    | Char(..)
                    | Lifetime(_)
                    | True
                    | False
                    | SelfLower
                    | SelfUpper
                    | Crate
                    | Super
                    | If
                    | Match
                    | Loop
                    | While
                    | For
                    | Unsafe
                    | Move
                    | Return
                    | Break
                    | Continue
                    | Let
                    | Const
                    | Static
                    | Async
                    | Minus
                    | Bang
                    | Star
                    | Amp
                    | AndAnd
                    | Pipe
                    | OrOr
                    | DotDot
                    | DotDotEq
                    | Lt
                    | Shl
                    | PathSep
                    | Pound
            ),
            _ => false,
        }
    }

    fn prefix_range(&mut self, r: R) -> Option<Ex> {
        let o = if self.eat(Tok::DotDot) {
            ".."
        } else {
            self.expect(Tok::DotDotEq)?;
            "..="
        };
        if !self.range_rhs(r) {
            return Some(Ex::new(o.into(), Prec::Range, EK::PrefixRange));
        }
        let e = self.assoc(Prec::LOr, R { stmt: false, ..r })?;
        let p = eff(&e) < Prec::LOr;
        Some(Ex::new(format!("{o}{}", wrap(e, p)?), Prec::Range, EK::PrefixRange))
    }

    fn unary(&mut self, r: R) -> Option<Ex> {
        let r2 = R { stmt: false, ..r };
        let op = match self.tok() {
            Some(Tok::Minus) => "-",
            Some(Tok::Bang) => "!",
            Some(Tok::Star) => "*",
            Some(Tok::Amp) => return self.addr_of(r2, ""),
            Some(Tok::AndAnd) => return self.addr_of(r2, "&"),
            _ => {
                let base = self.primary(r)?;
                return self.postfix(base, r);
            }
        };
        self.i += 1;
        let e = self.unary(r2)?;
        let p = eff(&e) < Prec::Prefix;
        Some(Ex::new(format!("{op}{}", wrap(e, p)?), Prec::Prefix, EK::Other))
    }

    /// `&x`, `&mut x`, `&raw const x`, `&raw mut x`; `outer` is the first
    /// `&` of a `&&` token.
    fn addr_of(&mut self, r: R, outer: &str) -> Option<Ex> {
        self.i += 1;
        let raw = matches!(self.peek(), Some(K::Tok(Tok::Ident(_))))
            && self.t[self.i].text == "raw"
            && matches!(self.tok_at(self.i + 1), Some(Tok::Const | Tok::Mut));
        let m = if raw {
            self.i += 1;
            if self.eat(Tok::Const) {
                "raw const "
            } else {
                self.bump();
                "raw mut "
            }
        } else if self.eat(Tok::Mut) {
            "mut "
        } else {
            ""
        };
        let e = self.unary(r)?;
        let p = eff(&e) < Prec::Prefix;
        Some(Ex::new(format!("{outer}&{m}{}", wrap(e, p)?), Prec::Prefix, EK::Other))
    }

    fn postfix(&mut self, mut e: Ex, r: R) -> Option<Ex> {
        loop {
            if self.eat(Tok::Question) {
                let p = e.prec < Prec::Unambiguous;
                e = Ex::new(format!("{}?", wrap(e, p)?), Prec::Unambiguous, EK::Other);
            } else if self.is(Tok::Dot) {
                self.i += 1;
                let p = e.prec < Prec::Unambiguous;
                let dot = if !p && e.kind == EK::DotLit { " ." } else { "." };
                let base = wrap(e, p)?;
                match self.tok()? {
                    Tok::Ident(_) | Tok::Reserved(_) => {
                        if matches!(self.tok(), Some(Tok::Reserved(_))) {
                            return None;
                        }
                        let name = self.bump();
                        let generics = if self.is(Tok::PathSep) {
                            self.i += 1;
                            format!("::{}", self.generic_args()?)
                        } else {
                            String::new()
                        };
                        if self.open(Delim::Paren) {
                            let args = self.call_args()?;
                            e = Ex::new(format!("{base}{dot}{name}{generics}({args})"), Prec::Unambiguous, EK::Other);
                        } else if generics.is_empty() {
                            e = Ex::new(format!("{base}{dot}{name}"), Prec::Unambiguous, EK::Field);
                        } else {
                            return None;
                        }
                    }
                    Tok::Int(..) | Tok::Float(_) => {
                        let name = self.bump();
                        e = Ex::new(format!("{base}{dot}{name}"), Prec::Unambiguous, EK::Other);
                    }
                    _ => return None,
                }
            } else if self.open(Delim::Paren) && !(r.stmt && e.block_like()) {
                let p = e.kind == EK::Field || e.prec < Prec::Unambiguous || e.block_like();
                let f = wrap(e, p)?;
                let args = self.call_args()?;
                e = Ex::new(format!("{f}({args})"), Prec::Unambiguous, EK::Other);
            } else if self.open(Delim::Bracket) && !(r.stmt && e.block_like()) {
                let p = e.prec < Prec::Unambiguous || e.block_like();
                let b = wrap(e, p)?;
                self.i += 1;
                let idx = self.expr(R::default())?;
                self.expect_close(Delim::Bracket)?;
                e = Ex::new(format!("{b}[{}]", idx.s), Prec::Unambiguous, EK::Other);
            } else {
                return Some(e);
            }
        }
    }

    /// `(a, b)` of a call: the text inside the parentheses.
    fn call_args(&mut self) -> Option<String> {
        self.i += 1;
        let (args, _) = self.list(Delim::Paren, |p| p.expr(R::default()).map(|e| e.s))?;
        Some(args.join(", "))
    }

    fn primary(&mut self, r: R) -> Option<Ex> {
        use Tok::*;
        let lit = |s: String| {
            let kind = if s.ends_with('.') { EK::DotLit } else { EK::Other };
            Some(Ex::new(s, Prec::Unambiguous, kind))
        };
        match self.peek()? {
            K::FragOpen(FragKind::Expr | FragKind::Expr2021 | FragKind::Literal) => {
                self.i += 1;
                let mut e = self.expr(R::default())?;
                (self.peek() == Some(K::FragClose)).then_some(())?;
                self.i += 1;
                e.frag = true;
                Some(e)
            }
            K::FragOpen(FragKind::Block) => {
                self.i += 1;
                let b = self.block()?;
                (self.peek() == Some(K::FragClose)).then_some(())?;
                self.i += 1;
                Some(Ex::new(b, Prec::Unambiguous, EK::Block))
            }
            K::FragOpen(FragKind::Path) => self.path_like(r),
            K::FragOpen(_) | K::FragClose | K::Close(_) => None,
            K::Open(Delim::Paren) => {
                self.i += 1;
                let (items, trailing) = self.list(Delim::Paren, |p| p.expr(R::default()).map(|e| e.s))?;
                if items.len() == 1 && !trailing {
                    return Some(Ex::new(format!("({})", items[0]), Prec::Unambiguous, EK::Other));
                }
                let one = if items.len() == 1 { "," } else { "" };
                Some(Ex::new(format!("({}{one})", items.join(", ")), Prec::Unambiguous, EK::Other))
            }
            K::Open(Delim::Bracket) => {
                self.i += 1;
                if self.close(Delim::Bracket) {
                    self.i += 1;
                    return Some(Ex::new("[]".into(), Prec::Unambiguous, EK::Other));
                }
                let first = self.expr(R::default())?;
                if self.eat(Semi) {
                    let n = self.expr(R::default())?;
                    self.expect_close(Delim::Bracket)?;
                    return Some(Ex::new(format!("[{}; {}]", first.s, n.s), Prec::Unambiguous, EK::Other));
                }
                let mut items = vec![first.s];
                if self.eat(Comma) {
                    let (rest, _) = self.list(Delim::Bracket, |p| p.expr(R::default()).map(|e| e.s))?;
                    items.extend(rest);
                } else {
                    self.expect_close(Delim::Bracket)?;
                }
                Some(Ex::new(format!("[{}]", items.join(", ")), Prec::Unambiguous, EK::Other))
            }
            K::Open(Delim::Brace) => Some(Ex::new(self.block()?, Prec::Unambiguous, EK::Block)),
            K::Tok(t) => match t {
                Int(..) | Float(_) | Str(..) | ByteStr(_) | ByteChar(_) | Char(..) | True | False => lit(self.bump()),
                Unsafe if self.kind(self.i + 1) == Some(K::Open(Delim::Brace)) => {
                    self.i += 1;
                    Some(Ex::new(format!("unsafe {}", self.block()?), Prec::Unambiguous, EK::BlockLike))
                }
                Lifetime(_) if self.tok_at(self.i + 1) == Some(Colon) => {
                    let label = self.bump();
                    self.i += 1;
                    let e = match self.peek()? {
                        K::Open(Delim::Brace) => Ex::new(self.block()?, Prec::Unambiguous, EK::Block),
                        K::Tok(Loop | While | For) => self.primary(r)?,
                        _ => return None,
                    };
                    Some(Ex::new(format!("{label}: {}", e.s), Prec::Unambiguous, e.kind))
                }
                If => self.if_expr(),
                Match => {
                    self.i += 1;
                    let scrut = cond(self.expr(R { no_struct: true, stmt: false })?)?;
                    if !self.open(Delim::Brace) {
                        return None;
                    }
                    self.i += 1;
                    let mut arms = String::new();
                    while !self.close(Delim::Brace) {
                        arms.push(' ');
                        arms.push_str(&self.arm()?);
                    }
                    self.i += 1;
                    let close = if arms.is_empty() { "}" } else { " }" };
                    Some(Ex::new(format!("match {scrut} {{{arms}{close}"), Prec::Unambiguous, EK::BlockLike))
                }
                Loop => {
                    self.i += 1;
                    Some(Ex::new(format!("loop {}", self.block()?), Prec::Unambiguous, EK::BlockLike))
                }
                While => {
                    self.i += 1;
                    let c = cond(self.expr(R { no_struct: true, stmt: false })?)?;
                    Some(Ex::new(format!("while {c} {}", self.block()?), Prec::Unambiguous, EK::BlockLike))
                }
                For => {
                    self.i += 1;
                    let pat = self.pat(true)?;
                    self.expect(In)?;
                    let iter = cond(self.expr(R { no_struct: true, stmt: false })?)?;
                    Some(Ex::new(format!("for {pat} in {iter} {}", self.block()?), Prec::Unambiguous, EK::BlockLike))
                }
                Pipe | OrOr | Move => self.closure(r),
                Break => {
                    self.i += 1;
                    let mut s = "break".to_string();
                    if matches!(self.tok(), Some(Lifetime(_))) {
                        s.push(' ');
                        s.push_str(&self.bump());
                    }
                    if self.range_rhs(r) {
                        // `break 'a: loop {}` needs parens rustc would add.
                        if !s.contains('\'') && matches!(self.tok(), Some(Lifetime(_))) {
                            return None;
                        }
                        let v = self.expr(R { stmt: false, ..r })?;
                        s.push(' ');
                        s.push_str(&wrap(v, false)?);
                        return Some(Ex::new(s, Prec::Jump, EK::Other));
                    }
                    Some(Ex::new(s, Prec::Unambiguous, EK::Other))
                }
                Continue => {
                    self.i += 1;
                    let mut s = "continue".to_string();
                    if matches!(self.tok(), Some(Lifetime(_))) {
                        s.push(' ');
                        s.push_str(&self.bump());
                    }
                    Some(Ex::new(s, Prec::Unambiguous, EK::Other))
                }
                Return => {
                    self.i += 1;
                    if self.range_rhs(r) {
                        let v = self.expr(R { stmt: false, ..r })?;
                        return Some(Ex::new(format!("return {}", wrap(v, false)?), Prec::Jump, EK::Other));
                    }
                    Some(Ex::new("return".into(), Prec::Unambiguous, EK::Other))
                }
                Let => {
                    self.i += 1;
                    let pat = self.pat(true)?;
                    self.expect(Eq)?;
                    let e = self.assoc(Prec::Compare, R { stmt: false, ..r })?;
                    let p = eff(&e) <= Prec::LAnd;
                    Some(Ex::new(format!("let {pat} = {}", wrap(e, p)?), Prec::Prefix, EK::Let))
                }
                Underscore => {
                    self.i += 1;
                    Some(Ex::new("_".into(), Prec::Unambiguous, EK::Other))
                }
                Ident(_) | SelfLower | SelfUpper | Super | Crate | PathSep | Lt | Shl => self.path_like(r),
                _ => None,
            },
        }
    }

    /// A path expression, a struct literal, or a macro call.
    fn path_like(&mut self, r: R) -> Option<Ex> {
        let start = self.i;
        let (path, generics) = self.path(true)?;
        if self.is(Tok::Bang) && !generics && matches!(self.kind(self.i + 1), Some(K::Open(_))) {
            self.i += 1;
            return Some(Ex::new(format!("{path}!{}", self.mac_group()?), Prec::Unambiguous, EK::Other));
        }
        if self.open(Delim::Brace) && !r.no_struct && self.kind(start) != Some(K::Tok(Tok::Lt)) {
            return Some(Ex::new(self.struct_lit(path)?, Prec::Unambiguous, EK::Other));
        }
        Some(Ex::new(path, Prec::Unambiguous, EK::Other))
    }

    /// `Path { a: 1, b, ..base }` after the path.
    fn struct_lit(&mut self, path: String) -> Option<String> {
        self.i += 1;
        let mut fields = Vec::new();
        let mut rest = None;
        while !self.close(Delim::Brace) {
            if self.eat(Tok::DotDot) {
                rest = Some(if self.close(Delim::Brace) { return None } else { self.expr(R::default())?.s });
                break;
            }
            let name = match self.tok()? {
                Tok::Ident(_) | Tok::Int(..) => self.bump(),
                _ => return None,
            };
            if self.eat(Tok::Colon) {
                fields.push(format!("{name}: {}", self.expr(R::default())?.s));
            } else {
                fields.push(name);
            }
            if !self.eat(Tok::Comma) && !self.close(Delim::Brace) {
                return None;
            }
        }
        self.expect_close(Delim::Brace)?;
        Some(match (fields.is_empty(), rest) {
            (true, None) => format!("{path} {{}}"),
            (true, Some(b)) => format!("{path} {{ ..{b} }}"),
            (false, None) => format!("{path} {{ {} }}", fields.join(", ")),
            (false, Some(b)) => format!("{path} {{ {}, ..{b} }}", fields.join(", ")),
        })
    }

    fn if_expr(&mut self) -> Option<Ex> {
        self.expect(Tok::If)?;
        let c = cond(self.expr(R { no_struct: true, stmt: false })?)?;
        let mut s = format!("if {c} {}", self.block()?);
        while self.eat(Tok::Else) {
            if self.is(Tok::If) {
                self.i += 1;
                let c = cond(self.expr(R { no_struct: true, stmt: false })?)?;
                s.push_str(&format!(" else if {c} {}", self.block()?));
            } else {
                s.push_str(&format!(" else {}", self.block()?));
                break;
            }
        }
        Some(Ex::new(s, Prec::Unambiguous, EK::BlockLike))
    }

    /// One match arm with pprust's comma.
    fn arm(&mut self) -> Option<String> {
        let pat = self.pat(true)?;
        let guard = if self.eat(Tok::If) { format!(" if {}", self.expr(R::default())?.s) } else { String::new() };
        self.expect(Tok::FatArrow)?;
        let body = self.expr(R { no_struct: false, stmt: true })?;
        if !self.eat(Tok::Comma) && !body.block_like() && !self.close(Delim::Brace) {
            return None;
        }
        let unsafe_block = body.kind == EK::BlockLike && body.s.starts_with("unsafe {");
        let comma = if body.kind != EK::Block || unsafe_block { "," } else { "" };
        Some(format!("{pat}{guard} => {}{comma}", body.s))
    }

    fn closure(&mut self, r: R) -> Option<Ex> {
        let mv = if self.eat(Tok::Move) { "move " } else { "" };
        let params = if self.eat(Tok::OrOr) {
            String::new()
        } else {
            self.expect(Tok::Pipe)?;
            let mut ps = Vec::new();
            while !self.is(Tok::Pipe) {
                let pat = self.pat(false)?;
                ps.push(if self.eat(Tok::Colon) { format!("{pat}: {}", self.ty()?) } else { pat });
                if !self.eat(Tok::Comma) && !self.is(Tok::Pipe) {
                    return None;
                }
            }
            self.i += 1;
            ps.join(", ")
        };
        if self.eat(Tok::Arrow) {
            let ty = self.ty()?;
            if !self.open(Delim::Brace) {
                return None;
            }
            let b = self.block()?;
            return Some(Ex::new(format!("{mv}|{params}| -> {ty} {b}"), Prec::Unambiguous, EK::Other));
        }
        let body = self.expr(R { stmt: false, ..r })?;
        Some(Ex::new(format!("{mv}|{params}| {}", body.s), Prec::Jump, EK::Other))
    }

    /// `{ stmts }` as pprust prints a block.
    fn block(&mut self) -> Option<String> {
        if !self.open(Delim::Brace) {
            return None;
        }
        self.i += 1;
        let mut stmts = Vec::new();
        while !self.close(Delim::Brace) {
            stmts.push(self.stmt()?);
        }
        self.i += 1;
        Some(if stmts.is_empty() { "{}".into() } else { format!("{{ {} }}", stmts.join(" ")) })
    }

    fn stmt(&mut self) -> Option<String> {
        use Tok::*;
        match self.peek()? {
            K::Tok(Semi) => {
                self.i += 1;
                return Some(";".into());
            }
            K::Tok(Let) => {
                self.i += 1;
                let mut s = format!("let {}", self.pat(true)?);
                if self.eat(Colon) {
                    s.push_str(&format!(": {}", self.ty()?));
                }
                if self.eat(Eq) {
                    let init = self.expr(R::default())?;
                    s.push_str(&format!(" = {}", init.s));
                    if self.eat(Else) {
                        if init.s.ends_with('}') {
                            return None;
                        }
                        s.push_str(&format!(" else {}", self.block()?));
                    }
                }
                self.expect(Semi)?;
                s.push(';');
                return Some(s);
            }
            // Items break the line in rustc's printer; attributes are not modeled.
            K::Tok(Fn | Struct | Enum | Impl | Trait | Use | Static | Mod | Type | Extern | Pub | Pound) => return None,
            K::Tok(Const) if self.kind(self.i + 1) != Some(K::Open(Delim::Brace)) => return None,
            _ => {}
        }
        // A macro call statement: `m! { .. }` and `m!(..);` stay statements.
        let start = self.i;
        if matches!(self.tok(), Some(Ident(_) | SelfLower | Super | Crate | PathSep)) {
            let (path, generics) = self.path(true)?;
            if !generics && self.is(Bang) && matches!(self.kind(self.i + 1), Some(K::Open(_))) {
                let brace = self.kind(self.i + 1) == Some(K::Open(Delim::Brace));
                self.i += 1;
                let mac = format!("{path}!{}", self.mac_group()?);
                if self.eat(Semi) {
                    return Some(format!("{mac};"));
                }
                if brace && !self.is(Dot) && !self.is(Question) {
                    return Some(mac);
                }
            }
            self.i = start;
        }
        let e = self.expr(R { no_struct: false, stmt: true })?;
        if self.eat(Semi) {
            return Some(format!("{};", e.s));
        }
        (self.close(Delim::Brace) || e.block_like()).then_some(e.s)
    }

    // ---------- paths ----------

    /// A path in expression or pattern position (`colons` true: generic
    /// args print as `::<..>`), or in a type (false). Also `<T as Tr>::x`.
    /// Returns the text and whether it holds generic args.
    fn path(&mut self, colons: bool) -> Option<(String, bool)> {
        let mut s = String::new();
        let mut generics = false;
        if self.peek() == Some(K::FragOpen(FragKind::Path)) {
            self.i += 1;
            let (p, g) = self.path(colons)?;
            (self.peek() == Some(K::FragClose)).then_some(())?;
            self.i += 1;
            s = p;
            generics = g;
        } else if self.eat(Tok::Lt) {
            let ty = self.ty()?;
            s.push('<');
            s.push_str(&ty);
            if self.eat(Tok::As) {
                let (tr, _) = self.path(false)?;
                s.push_str(" as ");
                s.push_str(&tr);
            }
            self.gt()?;
            s.push('>');
            self.expect(Tok::PathSep)?;
            s.push_str("::");
            generics |= self.segment(colons, &mut s)?;
        } else {
            if self.eat(Tok::PathSep) {
                s.push_str("::");
            }
            generics |= self.segment(colons, &mut s)?;
        }
        while self.is(Tok::PathSep) && !matches!(self.kind(self.i + 1), Some(K::Tok(Tok::Lt))) {
            self.i += 1;
            s.push_str("::");
            generics |= self.segment(colons, &mut s)?;
        }
        Some((s, generics))
    }

    /// One path segment and its generic args: `x::<T>` (expressions) or
    /// `X<T>` / `Fn(A) -> B` (types).
    fn segment(&mut self, colons: bool, s: &mut String) -> Option<bool> {
        match self.tok()? {
            Tok::Ident(_) | Tok::SelfLower | Tok::SelfUpper | Tok::Super | Tok::Crate => s.push_str(&self.bump()),
            _ => return None,
        }
        if s.ends_with("$crate") {
            return None;
        }
        let turbofish = self.is(Tok::PathSep) && self.tok_at(self.i + 1) == Some(Tok::Lt);
        if colons {
            if turbofish {
                self.i += 1;
                s.push_str("::");
                s.push_str(&self.generic_args()?);
                return Some(true);
            }
            return Some(false);
        }
        if turbofish {
            self.i += 1;
        }
        if self.is(Tok::Lt) {
            s.push_str(&self.generic_args()?);
            return Some(true);
        }
        if self.open(Delim::Paren) {
            // `Fn(A, B) -> C`.
            self.i += 1;
            let (args, _) = self.list(Delim::Paren, |p| p.ty())?;
            s.push_str(&format!("({})", args.join(", ")));
            if self.eat(Tok::Arrow) {
                s.push_str(&format!(" -> {}", self.ty()?));
            }
            return Some(true);
        }
        Some(false)
    }

    /// Consumes one `>` closing generic args (half of a `>>`).
    fn gt(&mut self) -> Option<()> {
        self.expect(Tok::Gt)
    }

    /// `<A, B, Item = C>` at the cursor.
    fn generic_args(&mut self) -> Option<String> {
        self.expect(Tok::Lt)?;
        let mut args = Vec::new();
        while !self.is(Tok::Gt) {
            let arg = match self.tok() {
                Some(Tok::Lifetime(_)) => self.bump(),
                Some(Tok::Ident(_)) if self.tok_at(self.i + 1) == Some(Tok::Eq) => {
                    let name = self.bump();
                    self.i += 1;
                    format!("{name} = {}", self.ty()?)
                }
                Some(Tok::Ident(_)) if self.tok_at(self.i + 1) == Some(Tok::Colon) => return None,
                Some(Tok::Int(..) | Tok::Minus | Tok::True | Tok::False | Tok::Char(..) | Tok::Str(..)) => self.unary(R::default())?.s,
                _ if self.open(Delim::Brace) => self.block()?,
                _ => self.ty()?,
            };
            args.push(arg);
            if !self.eat(Tok::Comma) && !self.is(Tok::Gt) {
                return None;
            }
        }
        self.gt()?;
        Some(format!("<{}>", args.join(", ")))
    }

    // ---------- types ----------

    fn ty(&mut self) -> Option<String> {
        use Tok::*;
        match self.peek()? {
            K::FragOpen(FragKind::Ty) => {
                self.i += 1;
                let t = self.ty()?;
                (self.peek() == Some(K::FragClose)).then_some(())?;
                self.i += 1;
                Some(t)
            }
            K::Open(Delim::Paren) => {
                self.i += 1;
                let (items, trailing) = self.list(Delim::Paren, |p| p.ty())?;
                Some(if items.len() == 1 && !trailing {
                    format!("({})", items[0])
                } else if items.len() == 1 {
                    format!("({},)", items[0])
                } else {
                    format!("({})", items.join(", "))
                })
            }
            K::Open(Delim::Bracket) => {
                self.i += 1;
                let t = self.ty()?;
                let s = if self.eat(Semi) { format!("[{t}; {}]", self.expr(R::default())?.s) } else { format!("[{t}]") };
                self.expect_close(Delim::Bracket)?;
                Some(s)
            }
            K::Tok(Amp | AndAnd) => {
                let outer = if self.is(AndAnd) { "&" } else { "" };
                self.i += 1;
                let lt = if matches!(self.tok(), Some(Lifetime(_))) { format!("{} ", self.bump()) } else { String::new() };
                let m = if self.eat(Mut) { "mut " } else { "" };
                Some(format!("{outer}&{lt}{m}{}", self.ty()?))
            }
            K::Tok(Star) => {
                self.i += 1;
                let m = if self.eat(Const) {
                    "const"
                } else {
                    self.expect(Mut)?;
                    "mut"
                };
                Some(format!("*{m} {}", self.ty()?))
            }
            K::Tok(Underscore) => {
                self.i += 1;
                Some("_".into())
            }
            K::Tok(Bang) => {
                self.i += 1;
                Some("!".into())
            }
            K::Tok(Fn) => {
                self.i += 1;
                if !self.open(Delim::Paren) {
                    return None;
                }
                self.i += 1;
                let (params, _) = self.list(Delim::Paren, |p| {
                    let named = matches!(p.tok(), Some(Ident(_) | Underscore)) && p.tok_at(p.i + 1) == Some(Colon);
                    if named {
                        let name = p.bump();
                        p.i += 1;
                        Some(format!("{name}: {}", p.ty()?))
                    } else {
                        p.ty()
                    }
                })?;
                let ret = if self.eat(Arrow) { format!(" -> {}", self.ty()?) } else { String::new() };
                Some(format!("fn({}){ret}", params.join(", ")))
            }
            K::Tok(Dyn) => {
                self.i += 1;
                Some(format!("dyn {}", self.bounds()?))
            }
            K::Tok(Impl) => {
                self.i += 1;
                Some(format!("impl {}", self.bounds()?))
            }
            K::Tok(Ident(_) | SelfUpper | SelfLower | Super | Crate | PathSep | Lt) => {
                let (p, generics) = self.path(false)?;
                if self.is(Bang) && !generics && matches!(self.kind(self.i + 1), Some(K::Open(_))) {
                    self.i += 1;
                    return Some(format!("{p}!{}", self.mac_group()?));
                }
                Some(p)
            }
            _ => None,
        }
    }

    /// `A + B + 'a` after `dyn` or `impl`.
    fn bounds(&mut self) -> Option<String> {
        let mut out = Vec::new();
        loop {
            let b = match self.tok()? {
                Tok::Lifetime(_) => self.bump(),
                Tok::Question => {
                    self.i += 1;
                    format!("?{}", self.path(false)?.0)
                }
                _ => self.path(false)?.0,
            };
            out.push(b);
            if !self.eat(Tok::Plus) {
                return Some(out.join(" + "));
            }
        }
    }

    // ---------- patterns ----------

    /// A pattern; `alt` allows a top-level `a | b` (and a leading `|`).
    fn pat(&mut self, alt: bool) -> Option<String> {
        if !alt {
            return self.pat1();
        }
        self.eat(Tok::Pipe);
        let mut out = vec![self.pat1()?];
        while self.eat(Tok::Pipe) {
            out.push(self.pat1()?);
        }
        Some(out.join(" | "))
    }

    fn pat1(&mut self) -> Option<String> {
        use Tok::*;
        let k = self.peek()?;
        let s = match k {
            K::FragOpen(FragKind::Pat | FragKind::PatParam) => {
                self.i += 1;
                let p = self.pat(true)?;
                (self.peek() == Some(K::FragClose)).then_some(())?;
                self.i += 1;
                return Some(p);
            }
            K::Open(Delim::Paren) => {
                self.i += 1;
                let (items, trailing) = self.list(Delim::Paren, |p| p.pat(true))?;
                return Some(if items.len() == 1 && !trailing && items[0] != ".." {
                    format!("({})", items[0])
                } else if items.len() == 1 {
                    format!("({},)", items[0])
                } else {
                    format!("({})", items.join(", "))
                });
            }
            K::Open(Delim::Bracket) => {
                self.i += 1;
                let (items, _) = self.list(Delim::Bracket, |p| p.pat(true))?;
                return Some(format!("[{}]", items.join(", ")));
            }
            K::Tok(Underscore) => {
                self.i += 1;
                return Some("_".into());
            }
            K::Tok(DotDot) if matches!(self.kind(self.i + 1), Some(K::Tok(Comma) | K::Close(_) | K::Tok(Pipe) | K::Tok(FatArrow))) => {
                self.i += 1;
                return Some("..".into());
            }
            K::Tok(DotDotEq | DotDot) => {
                let o = self.bump();
                return Some(format!("{o}{}", self.range_end()?));
            }
            K::Tok(Amp | AndAnd) => {
                let outer = if self.is(AndAnd) { "&" } else { "" };
                self.i += 1;
                let m = if self.eat(Mut) { "mut " } else { "" };
                let inner = self.pat1()?;
                return Some(format!("{outer}&{m}{inner}"));
            }
            K::Tok(Ref | Mut) => {
                let mut s = String::new();
                if self.eat(Ref) {
                    s.push_str("ref ");
                }
                if self.eat(Mut) {
                    s.push_str("mut ");
                }
                if s == "mut " && self.is(Ref) {
                    return None;
                }
                match self.tok()? {
                    Ident(_) => s.push_str(&self.bump()),
                    _ => return None,
                }
                return self.binding_tail(s);
            }
            K::Tok(Ident(_))
                if !matches!(
                    self.kind(self.i + 1),
                    Some(K::Tok(PathSep | Bang | DotDot | DotDotEq | Ellipsis)) | Some(K::Open(Delim::Paren | Delim::Brace))
                ) =>
            {
                let name = self.bump();
                return self.binding_tail(name);
            }
            K::Tok(Int(..) | Float(_) | Str(..) | ByteStr(_) | ByteChar(_) | Char(..) | True | False) => self.bump(),
            K::Tok(Minus) if matches!(self.tok_at(self.i + 1), Some(Int(..) | Float(_))) => {
                self.i += 1;
                format!("-{}", self.bump())
            }
            K::Tok(Ident(_) | SelfLower | SelfUpper | Super | Crate | PathSep | Lt) => {
                let (path, generics) = self.path(true)?;
                if self.is(Bang) && !generics && matches!(self.kind(self.i + 1), Some(K::Open(_))) {
                    self.i += 1;
                    return Some(format!("{path}!{}", self.mac_group()?));
                }
                if self.open(Delim::Paren) {
                    self.i += 1;
                    let (items, _) = self.list(Delim::Paren, |p| p.pat(true))?;
                    return Some(format!("{path}({})", items.join(", ")));
                }
                if self.open(Delim::Brace) {
                    return self.struct_pat(path);
                }
                path
            }
            _ => return None,
        };
        // A literal or path may start a range pattern.
        if self.is(DotDotEq) || self.is(Ellipsis) {
            let o = self.bump();
            return Some(format!("{s}{o}{}", self.range_end()?));
        }
        if self.is(DotDot) {
            self.i += 1;
            let end = if matches!(self.peek(), Some(K::Tok(Comma | Pipe | FatArrow | If | Eq | Colon)) | Some(K::Close(_)) | None) {
                String::new()
            } else {
                self.range_end()?
            };
            return Some(format!("{s}..{end}"));
        }
        Some(s)
    }

    fn range_end(&mut self) -> Option<String> {
        use Tok::*;
        match self.tok()? {
            Int(..) | Float(_) | Char(..) | ByteChar(_) => Some(self.bump()),
            Minus => {
                self.i += 1;
                match self.tok()? {
                    Int(..) | Float(_) => Some(format!("-{}", self.bump())),
                    _ => None,
                }
            }
            Ident(_) | SelfUpper | PathSep | Lt | Crate | Super | SelfLower => Some(self.path(true)?.0),
            _ => None,
        }
    }

    /// `name @ sub` after a binding name.
    fn binding_tail(&mut self, name: String) -> Option<String> {
        if !self.eat(Tok::At) {
            return Some(name);
        }
        let sub = self.pat1()?;
        Some(format!("{name} @ {sub}"))
    }

    fn struct_pat(&mut self, path: String) -> Option<String> {
        self.i += 1;
        let mut fields = Vec::new();
        let mut rest = false;
        while !self.close(Delim::Brace) {
            if self.eat(Tok::DotDot) {
                rest = true;
                break;
            }
            let field = match self.tok()? {
                Tok::Ident(_) | Tok::Int(..) if self.tok_at(self.i + 1) == Some(Tok::Colon) => {
                    let name = self.bump();
                    self.i += 1;
                    format!("{name}: {}", self.pat(true)?)
                }
                Tok::Ref | Tok::Mut | Tok::Ident(_) => self.pat1()?,
                _ => return None,
            };
            fields.push(field);
            if !self.eat(Tok::Comma) && !self.close(Delim::Brace) {
                return None;
            }
        }
        self.expect_close(Delim::Brace)?;
        Some(match (fields.is_empty(), rest) {
            (true, false) => format!("{path} {{}}"),
            (true, true) => format!("{path} {{ .. }}"),
            (false, false) => format!("{path} {{ {} }}", fields.join(", ")),
            (false, true) => format!("{path} {{ {}, .. }}", fields.join(", ")),
        })
    }

    // ---------- macro calls ----------

    /// The delimited arguments of a macro call at the cursor, as pprust's
    /// `print_mac` writes them after the `!`.
    fn mac_group(&mut self) -> Option<String> {
        let open = self.i;
        let close = self.group_end(open)?;
        self.i = close + 1;
        let K::Open(d) = self.t[open].k else { return None };
        let inner = self.tts(open + 1, close)?;
        Some(match d {
            Delim::Brace if inner.is_empty() => " {}".into(),
            Delim::Brace => format!(" {{ {inner} }}"),
            Delim::Paren => format!("({inner})"),
            Delim::Bracket => format!("[{inner}]"),
        })
    }

    /// The index of the token that closes the group opened at `open`.
    fn group_end(&self, open: usize) -> Option<usize> {
        let mut depth = 0i32;
        for j in open..self.t.len() {
            match self.t[j].k {
                K::Open(_) | K::FragOpen(_) => depth += 1,
                K::Close(_) | K::FragClose => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(j);
                    }
                }
                K::Tok(_) => {}
            }
        }
        None
    }

    /// rustc's `print_tts` over the token trees in `lo..hi`.
    fn tts(&self, lo: usize, hi: usize) -> Option<String> {
        // Each tree: its first and last token index.
        let mut trees = Vec::new();
        let mut j = lo;
        while j < hi {
            let end = match self.t[j].k {
                K::Open(_) | K::FragOpen(_) => self.group_end(j)?,
                K::Tok(Tok::DocStr(_)) => return None,
                K::Tok(_) => j,
                K::Close(_) | K::FragClose => return None,
            };
            trees.push((j, end));
            j = end + 1;
        }
        let mut s = String::new();
        for (n, &(a, b)) in trees.iter().enumerate() {
            s.push_str(&self.tree_text(a, b)?);
            if let Some(&(c, _)) = trees.get(n + 1)
                && self.space_after(a, b, c)?
            {
                s.push(' ');
            }
        }
        Some(s)
    }

    fn tree_text(&self, a: usize, b: usize) -> Option<String> {
        let t = &self.t[a];
        Some(match t.k {
            K::Tok(_) if t.text.starts_with("$crate") => return None,
            K::Tok(_) => t.text.clone(),
            K::FragOpen(_) => self.tts(a + 1, b)?,
            K::Open(Delim::Brace) => {
                let inner = self.tts(a + 1, b)?;
                if inner.is_empty() {
                    "{}".into()
                } else if t.alone? {
                    format!("{{ {inner} }}")
                } else {
                    format!("{{{inner}}}")
                }
            }
            K::Open(d) => {
                let (o, c) = if d == Delim::Paren { ("(", ")") } else { ("[", "]") };
                format!("{o}{}{c}", self.tts(a + 1, b)?)
            }
            K::Close(_) | K::FragClose => return None,
        })
    }

    /// Whether `print_tts` puts a space between the tree `a..=b` and the
    /// tree starting at `c`. None when it depends on what lust lost.
    fn space_after(&self, a: usize, b: usize, c: usize) -> Option<bool> {
        let one = |nt: bool, alone: bool, next_nt: bool| {
            if alone { self.space_between(a, b, c, nt, next_nt) } else { a == b && self.ident_tok(a) && self.ident_tok(c) }
        };
        let alone = self.t[b].alone;
        let (sa, sc) = (self.t[a].subst && a == b, self.t[c].subst);
        // A substituted name: an `$x:ident` (NtIdent, `Alone`) or an `$x:tt`.
        let mut answers = Vec::new();
        for nt in if sa { vec![true, false] } else { vec![false] } {
            for next_nt in if sc { vec![true, false] } else { vec![false] } {
                let al = if nt { true } else { alone? };
                answers.push(one(nt, al, next_nt));
            }
        }
        answers.iter().all(|&x| x == answers[0]).then(|| answers[0])
    }

    fn ident_tok(&self, i: usize) -> bool {
        matches!(self.t[i].k, K::Tok(t) if ident_like(t))
    }

    /// rustc's `space_between` for the trees `a..=b` and the one at `c`.
    /// `nt`: the first tree is an NtIdent, which the ident rules skip.
    fn space_between(&self, a: usize, b: usize, c: usize, nt: bool, next_nt: bool) -> bool {
        let first = if a == b { self.tok_at(a) } else { None };
        let next = match self.t[c].k {
            K::Tok(t) => Some(t),
            _ => None,
        };
        let punct = |t: Option<Tok>| t.is_some_and(is_punct);
        let raw = self.t[a].text.starts_with("r#");
        // A non-reserved identifier: a plain name or a raw one.
        let plain_ident = !nt && matches!(first, Some(Tok::Ident(_))) || !nt && raw;
        match (first, next) {
            (Some(Tok::Dot), _) if !punct(next) => false,
            (Some(Tok::Dollar), Some(t)) if ident_like(t) && !next_nt => false,
            (_, Some(Tok::Comma | Tok::Semi | Tok::Dot)) if !punct(first) => false,
            (_, Some(Tok::Bang)) if plain_ident => false,
            (_, None)
                if self.t[c].k == K::Open(Delim::Paren)
                    && (plain_ident || !nt && matches!(first, Some(Tok::Fn | Tok::SelfUpper | Tok::Pub))) =>
            {
                false
            }
            (Some(Tok::Pound), None) if self.t[c].k == K::Open(Delim::Bracket) => false,
            _ => true,
        }
    }
}
