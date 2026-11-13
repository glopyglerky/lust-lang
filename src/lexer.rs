// Lexer: turns source bytes into a flat token list. Identifiers are interned here.
use crate::diag::{Diag, FileId, Span};
use crate::intern::{FxMap, Interner, Sym};
use std::borrow::Cow;

mod xid_table {
    include!("xid_table.rs");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntSuffix {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
    I128,
    U128,
}

impl IntSuffix {
    /// The suffix as written (`u8`).
    pub fn name(self) -> &'static str {
        match self {
            IntSuffix::I8 => "i8",
            IntSuffix::I16 => "i16",
            IntSuffix::I32 => "i32",
            IntSuffix::I64 => "i64",
            IntSuffix::Isize => "isize",
            IntSuffix::U8 => "u8",
            IntSuffix::U16 => "u16",
            IntSuffix::U32 => "u32",
            IntSuffix::U64 => "u64",
            IntSuffix::Usize => "usize",
            IntSuffix::I128 => "i128",
            IntSuffix::U128 => "u128",
        }
    }
}

/// Literal tokens keep their written text, because rustc compares
/// literal tokens in macro matchers by it (`0x1` is not `1`) and
/// `stringify!` prints it. `Int`, `Str`, and `Char` carry the value the
/// parser reads plus an optional text index into `Lexed::strings`: None
/// when the text is the canonical form `lit_text` prints from the value
/// (`1`, `"a"`, `'a'`), which is also the text of a token a builtin
/// macro makes. `Float`, `ByteStr`, and `ByteChar` hold only their text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tok {
    Ident(Sym),
    /// A Rust 2024 keyword that cannot be used as an identifier (`gen`, `try`, `await`, and kin).
    /// The parser rejects it wherever an identifier is expected, with a clearer message.
    Reserved(Sym),
    /// Value, suffix, and written text (see above).
    Int(u128, Option<IntSuffix>, Option<u32>),
    /// Index into `Lexed::strings`, holding the literal text (the parser rejects floats for now).
    Float(u32),
    /// Indexes into `Lexed::strings`: the value, and the written text (see above).
    Str(u32, Option<u32>),
    /// The string of a doc comment (`///`, `/** */`, and kin). The lexer
    /// emits a doc comment as the `#[doc = ...]` tokens it stands for in
    /// rustc's token trees; this is the literal's index into
    /// `Lexed::strings`, holding the comment text. `stringify!` prints it
    /// as the raw string rustc prints (`r" hi"`).
    DocStr(u32),
    /// A byte string (`b".."`, `br".."`) or C string (`c".."`, `cr".."`) literal.
    /// Index into `Lexed::strings`, holding the token's verbatim source text (prefix
    /// and quotes included). The parser rejects the form before any value is needed;
    /// the text is what macro token matching and `stringify!` compare and print, so
    /// two byte literals with different bytes must not look equal here.
    ByteStr(u32),
    /// A byte char literal (`b'x'`). Like `ByteStr`, the index holds verbatim source text.
    ByteChar(u32),
    /// Value and written text (see above).
    Char(char, Option<u32>),
    /// A lifetime or loop label name, without the leading `'` (`'a`, `'static`, `'_`).
    Lifetime(Sym),
    // Keywords.
    As,
    Break,
    Const,
    Continue,
    Else,
    Enum,
    False,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Mut,
    Pub,
    Ref,
    Return,
    SelfLower,
    SelfUpper,
    Static,
    Struct,
    Trait,
    True,
    Type,
    Unsafe,
    Use,
    Where,
    While,
    Async,
    Dyn,
    Move,
    Crate,
    Super,
    Extern,
    Await,
    // Punctuation.
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semi,
    Colon,
    PathSep,
    Dot,
    DotDot,
    DotDotEq,
    /// `...`: one token tree in rustc, so macro matching and tt
    /// counting must see it whole, not as `..` plus `.`.
    Ellipsis,
    Arrow,
    FatArrow,
    Eq,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    AmpEq,
    PipeEq,
    CaretEq,
    ShlEq,
    Amp,
    AndAnd,
    Pipe,
    OrOr,
    Caret,
    Bang,
    Pound,
    Question,
    Underscore,
    Shl,
    /// `>>` as one token tree. The lexer never emits this from
    /// source (it emits two `Gt` so generics parse without
    /// splitting); the expander merges two touching `Gt` into one
    /// `Shr` when it builds token trees, because rustc's token
    /// trees hold `>>` whole, and splits it again before the
    /// parser. `ShrEq` is the same for `>>=`.
    Shr,
    ShrEq,
    Dollar,
    At,
    Eof,
}

impl Tok {
    /// The `Lexed::strings` indexes this token carries, writable: `Float`,
    /// `DocStr`, `ByteStr`, and `ByteChar` hold one, `Str` a value and
    /// maybe a text, `Int` and `Char` maybe a text. This is the one list
    /// of such kinds. Every pass that moves token streams between string
    /// tables (concat, splice, slice) or hashes their strings goes
    /// through it, so a new kind is added here only.
    pub fn string_indices_mut(&mut self) -> [Option<&mut u32>; 2] {
        match self {
            Tok::Float(i) | Tok::DocStr(i) | Tok::ByteStr(i) | Tok::ByteChar(i) => [Some(i), None],
            Tok::Str(i, text) => [Some(i), text.as_mut()],
            Tok::Int(_, _, text) | Tok::Char(_, text) => [text.as_mut(), None],
            _ => [None, None],
        }
    }

    /// The `Lexed::strings` indexes this token carries (see `string_indices_mut`).
    pub fn string_indices(mut self) -> impl Iterator<Item = u32> {
        self.string_indices_mut().map(|i| i.map(|i| *i)).into_iter().flatten()
    }

    /// Adds `offset` to every `Lexed::strings` index this token carries:
    /// the token moved to a table whose strings start `offset` later.
    pub fn shift_strings(&mut self, offset: u32) {
        for i in self.string_indices_mut().into_iter().flatten() {
            *i = i.wrapping_add(offset);
        }
    }
}

/// The written text of a literal token (`0x1`, `'\x61'`, `r"a"`, a doc
/// comment's `r" hi"`), or None for any other token. rustc's macro
/// matcher compares literal tokens by this text, and `stringify!` prints
/// it. A token with no text index prints canonically from its value.
pub fn lit_text<'s>(tok: &Tok, strings: &'s [String]) -> Option<std::borrow::Cow<'s, str>> {
    use std::borrow::Cow;
    let at = |i: u32| strings.get(i as usize).map_or("", String::as_str);
    Some(match *tok {
        Tok::Int(_, _, Some(t)) | Tok::Str(_, Some(t)) | Tok::Char(_, Some(t)) | Tok::Float(t) | Tok::ByteStr(t) | Tok::ByteChar(t) => {
            Cow::Borrowed(at(t))
        }
        Tok::Int(v, suffix, None) => Cow::Owned(format!("{v}{}", suffix.map_or("", IntSuffix::name))),
        // rustc's form for a string it makes (`escape_string_symbol`).
        Tok::Str(i, None) => Cow::Owned(format!("\"{}\"", at(i).escape_debug())),
        Tok::Char(c, None) => Cow::Owned(format!("{c:?}")),
        // A doc comment's literal is the raw string rustc uses for it in
        // token streams, with enough `#`s to close safely.
        Tok::DocStr(i) => {
            let text = at(i);
            let mut hashes = 0usize;
            let bytes = text.as_bytes();
            for (j, &b) in bytes.iter().enumerate() {
                if b == b'"' {
                    let run = bytes[j + 1..].iter().take_while(|&&c| c == b'#').count();
                    hashes = hashes.max(run + 1);
                }
            }
            let marks = "#".repeat(hashes);
            Cow::Owned(format!("r{marks}\"{text}\"{marks}"))
        }
        _ => return None,
    })
}

/// A delimiter kind for token trees.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delim {
    Paren,
    Bracket,
    Brace,
}

/// One token tree: a single token, or a delimited group of trees. Macro calls and
/// attributes in the AST hold token trees; the expander (docs/ARCH.md section 6) is
/// their first consumer. The parser builds trees only for the shapes it stores.
#[allow(dead_code)] // Delimiter kinds and spans are first read by macro expansion.
#[derive(Clone, Debug)]
pub enum TokenTree {
    Token(Token),
    Delim(Delim, Vec<TokenTree>, Span),
}

#[derive(Clone, Copy, Debug)]
pub struct Token {
    pub tok: Tok,
    pub span: Span,
}

/// Groups a flat token slice into nested token trees, skipping a final `Eof`
/// token. Returns the trees and true when every delimiter was balanced. On
/// unbalanced input the trees nest on a best-effort basis (an unclosed group
/// runs to the end) and the flag is false; callers that need balance check it.
pub fn build_trees(toks: &[Token]) -> (Vec<TokenTree>, bool) {
    fn open(t: Tok) -> Option<Delim> {
        match t {
            Tok::LParen => Some(Delim::Paren),
            Tok::LBracket => Some(Delim::Bracket),
            Tok::LBrace => Some(Delim::Brace),
            _ => None,
        }
    }
    fn closes(d: Delim, t: Tok) -> bool {
        matches!((d, t), (Delim::Paren, Tok::RParen) | (Delim::Bracket, Tok::RBracket) | (Delim::Brace, Tok::RBrace))
    }
    fn is_close(t: Tok) -> bool {
        matches!(t, Tok::RParen | Tok::RBracket | Tok::RBrace)
    }
    // One stack frame per open delimiter: its kind, opening span, and trees so far.
    let mut stack: Vec<(Delim, Span, Vec<TokenTree>)> = Vec::new();
    let mut out: Vec<TokenTree> = Vec::new();
    let mut balanced = true;
    for t in toks {
        if t.tok == Tok::Eof {
            break;
        }
        if let Some(d) = open(t.tok) {
            stack.push((d, t.span, Vec::new()));
            continue;
        }
        if is_close(t.tok) {
            match stack.last() {
                Some((d, _, _)) if closes(*d, t.tok) => {
                    let (d, span, inner) = stack.pop().unwrap();
                    // A group's span covers both delimiters, so spacing
                    // readers (`stringify!`) can see where it ends.
                    let tree = TokenTree::Delim(d, inner, span.to(t.span));
                    match stack.last_mut() {
                        Some((_, _, parent)) => parent.push(tree),
                        None => out.push(tree),
                    }
                }
                _ => {
                    balanced = false;
                    match stack.last_mut() {
                        Some((_, _, parent)) => parent.push(TokenTree::Token(*t)),
                        None => out.push(TokenTree::Token(*t)),
                    }
                }
            }
            continue;
        }
        match stack.last_mut() {
            Some((_, _, parent)) => parent.push(TokenTree::Token(*t)),
            None => out.push(TokenTree::Token(*t)),
        }
    }
    if !stack.is_empty() {
        balanced = false;
        // Close what is left, innermost first, so no token is dropped.
        while let Some((d, span, inner)) = stack.pop() {
            let tree = TokenTree::Delim(d, inner, span);
            match stack.last_mut() {
                Some((_, _, parent)) => parent.push(tree),
                None => out.push(tree),
            }
        }
    }
    (out, balanced)
}

/// Flattens token trees back into a flat token list, re-creating the closing
/// delimiter tokens from each group's kind and span. The worklist path
/// (ARCH 6.1) reads this when it owns expansion; today's expander
/// flattens its own tree type.
#[allow(dead_code)]
pub fn flatten_trees(trees: &[TokenTree], out: &mut Vec<Token>) {
    for tree in trees {
        match tree {
            TokenTree::Token(t) => out.push(*t),
            TokenTree::Delim(d, inner, span) => {
                let (open, close) = match d {
                    Delim::Paren => (Tok::LParen, Tok::RParen),
                    Delim::Bracket => (Tok::LBracket, Tok::RBracket),
                    Delim::Brace => (Tok::LBrace, Tok::RBrace),
                };
                out.push(Token { tok: open, span: *span });
                flatten_trees(inner, out);
                out.push(Token { tok: close, span: *span });
            }
        }
    }
}

/// A token stream packed for a cache that keeps it across builds (resolve/worklist.rs
/// `ResolveCache`, which a front image saves and restores each build): each distinct token
/// once (`palette`), then per token, as LEB128 varints, its palette index (doubled, plus one
/// when its file or context differs from the last token's, which then follow), its start as a
/// zigzag delta from the last token's start, and its length. Tokens repeat and spans climb, so
/// a stream packs to a fraction of `Lexed`'s 48 bytes a token.
#[derive(Clone, Default)]
pub struct PackedLexed {
    palette: Vec<Tok>,
    codes: Vec<u8>,
    len: usize,
    strings: Vec<String>,
}

impl PackedLexed {
    pub fn pack(l: &Lexed) -> PackedLexed {
        fn put(out: &mut Vec<u8>, mut v: u64) {
            while v >= 0x80 {
                out.push(v as u8 | 0x80);
                v >>= 7;
            }
            out.push(v as u8);
        }
        let zig = |d: i64| ((d << 1) ^ (d >> 63)) as u64;
        let mut index: FxMap<Tok, u64> = FxMap::default();
        let mut palette: Vec<Tok> = Vec::new();
        let mut codes = Vec::with_capacity(l.tokens.len() * 4);
        let (mut file, mut ctxt, mut lo) = (FileId(0), crate::diag::CtxtId(0), 0u32);
        for t in &l.tokens {
            let i = *index.entry(t.tok).or_insert_with(|| {
                palette.push(t.tok);
                palette.len() as u64 - 1
            });
            let moved = t.span.file != file || t.span.ctxt != ctxt;
            put(&mut codes, i << 1 | moved as u64);
            if moved {
                (file, ctxt) = (t.span.file, t.span.ctxt);
                put(&mut codes, file.0 as u64);
                put(&mut codes, ctxt.0 as u64);
            }
            put(&mut codes, zig(t.span.lo as i64 - lo as i64));
            put(&mut codes, zig(t.span.hi as i64 - t.span.lo as i64));
            lo = t.span.lo;
        }
        codes.shrink_to_fit();
        PackedLexed { palette, codes, len: l.tokens.len(), strings: l.strings.clone() }
    }

    /// The stream back, token for token.
    pub fn unpack(&self) -> Lexed {
        let get = |at: &mut usize| {
            let (mut v, mut shift) = (0u64, 0);
            while let Some(&b) = self.codes.get(*at) {
                *at += 1;
                v |= ((b & 0x7f) as u64) << shift;
                if b < 0x80 {
                    break;
                }
                shift += 7;
            }
            v
        };
        let unzig = |v: u64| (v >> 1) as i64 ^ -((v & 1) as i64);
        let mut tokens = Vec::with_capacity(self.len);
        let (mut at, mut file, mut ctxt, mut lo) = (0, FileId(0), crate::diag::CtxtId(0), 0u32);
        for _ in 0..self.len {
            let c = get(&mut at);
            if c & 1 == 1 {
                file = FileId(get(&mut at) as u32);
                ctxt = crate::diag::CtxtId(get(&mut at) as u32);
            }
            lo = (lo as i64 + unzig(get(&mut at))) as u32;
            let hi = (lo as i64 + unzig(get(&mut at))) as u32;
            tokens.push(Token { tok: self.palette[(c >> 1) as usize], span: Span { file, lo, hi, ctxt } });
        }
        Lexed { tokens, strings: self.strings.clone() }
    }
}

#[derive(Clone)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub strings: Vec<String>,
}

fn keyword(s: &str) -> Option<Tok> {
    Some(match s {
        "as" => Tok::As,
        "break" => Tok::Break,
        "const" => Tok::Const,
        "continue" => Tok::Continue,
        "else" => Tok::Else,
        "enum" => Tok::Enum,
        "false" => Tok::False,
        "fn" => Tok::Fn,
        "for" => Tok::For,
        "if" => Tok::If,
        "impl" => Tok::Impl,
        "in" => Tok::In,
        "let" => Tok::Let,
        "loop" => Tok::Loop,
        "match" => Tok::Match,
        "mod" => Tok::Mod,
        "mut" => Tok::Mut,
        "pub" => Tok::Pub,
        "ref" => Tok::Ref,
        "return" => Tok::Return,
        "self" => Tok::SelfLower,
        "Self" => Tok::SelfUpper,
        "static" => Tok::Static,
        "struct" => Tok::Struct,
        "trait" => Tok::Trait,
        "true" => Tok::True,
        "type" => Tok::Type,
        "unsafe" => Tok::Unsafe,
        "use" => Tok::Use,
        "where" => Tok::Where,
        "while" => Tok::While,
        "async" => Tok::Async,
        "dyn" => Tok::Dyn,
        "move" => Tok::Move,
        "crate" => Tok::Crate,
        "super" => Tok::Super,
        "extern" => Tok::Extern,
        "await" => Tok::Await,
        "_" => Tok::Underscore,
        _ => return None,
    })
}

/// Keywords of Rust 2024 (strict and reserved) that have no token of their own here.
/// Weak keywords such as `union`, `safe`, and `macro_rules` stay usable as identifiers, as in rustc.
/// `crate`, `super`, `extern`, and `await` have their own tokens so the parser can
/// recognize paths, extern items, and `.await` and reject them with the right code.
fn reserved(s: &str) -> bool {
    matches!(
        s,
        "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "gen"
            | "macro"
            | "override"
            | "priv"
            | "try"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
    )
}

/// Names that rustc does not allow even as raw identifiers (`r#self` and friends).
fn raw_ident_invalid(s: &str) -> bool {
    matches!(s, "self" | "Self" | "super" | "crate" | "_")
}

/// True when a lifetime token named `name` was written raw (`'r#fn`): the
/// token keeps only the name, and its span also covers the `r#`.
pub fn lifetime_is_raw(name: &str, span: Span) -> bool {
    span.hi.saturating_sub(span.lo) as usize == name.len() + 3
}

/// The error rustc's parser reports for a lifetime (or, with `label`, a
/// loop label) named by a keyword, such as `'fn` or `'gen`, or None when
/// the name is fine. A raw name (`'r#fn`) is fine, and so are the
/// lifetimes `'_` and `'static` (not the labels). Token trees may hold
/// such names (`stringify!('fn)`): the error is for parsed lifetime and
/// label positions only, which call this.
pub fn keyword_lifetime_error(name: &str, raw: bool, label: bool, span: Span) -> Option<Diag> {
    if raw || (keyword(name).is_none() && !reserved(name)) || (!label && matches!(name, "_" | "static")) {
        return None;
    }
    let what = if label { "labels" } else { "lifetimes" };
    Some(Diag::new("L0001", span, format!("{what} cannot use keyword names"), format!("Rename `'{name}`, for example to `'a`.")))
}

/// True when code point `cp` is in one of the sorted ranges.
fn in_ranges(ranges: &[(u32, u32)], cp: u32) -> bool {
    ranges
        .binary_search_by(|&(lo, hi)| {
            if cp < lo {
                std::cmp::Ordering::Greater
            } else if cp > hi {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// True when `c` has the Unicode XID_Start property (Unicode 17, as
/// rustc 1.99). Callers add `_`, which Rust allows but XID_Start
/// does not. ASCII answers come from the first two table ranges.
#[inline]
fn is_xid_start(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphabetic();
    }
    in_ranges(xid_table::XID_START_RANGES, c as u32)
}

/// True when `c` has the Unicode XID_Continue property. Every
/// XID_Start char is also XID_Continue.
#[inline]
fn is_xid_continue(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphanumeric() || c == '_';
    }
    in_ranges(xid_table::XID_CONTINUE_RANGES, c as u32)
}

/// True when an identifier can start at byte `at` of `src`: an ASCII
/// letter or `_`, or a non-ASCII XID_Start char. The ASCII cases
/// answer from the byte alone; only a byte at or above 0x80 decodes
/// a char and looks in the XID table.
#[inline]
fn is_ident_start_at(src: &str, b: &[u8], at: usize) -> bool {
    match b.get(at) {
        Some(&c) if c.is_ascii_alphabetic() || c == b'_' => true,
        Some(&c) if c >= 0x80 => src[at..].chars().next().is_some_and(is_xid_start),
        _ => false,
    }
}

/// Scans identifier chars from `*i`, which must be at a valid start
/// char. ASCII letters, digits, and `_` scan in a byte loop in a
/// local index, with no char decode and no table lookup, so an
/// ASCII name costs one branch per byte. Only a byte at or above
/// 0x80 goes to the char loop. Returns true when the scanned text
/// holds a non-ASCII char.
#[inline]
fn scan_ident_tail(src: &str, b: &[u8], i: &mut usize) -> bool {
    let mut j = *i;
    while j < b.len() {
        let c = b[j];
        if c.is_ascii_alphanumeric() || c == b'_' {
            j += 1;
        } else {
            break;
        }
    }
    *i = j;
    if j >= b.len() || b[j] < 0x80 {
        return false;
    }
    scan_ident_tail_unicode(src, b, i)
}

/// Continues an identifier scan at `*i`, which is at a non-ASCII
/// char, with a char loop that stops at the first char that is not
/// XID_Continue (rustc ends the name there and then rejects that
/// char on its own). Always returns true: the name is non-ASCII.
fn scan_ident_tail_unicode(src: &str, b: &[u8], i: &mut usize) -> bool {
    let mut j = *i;
    while j < b.len() {
        let c = b[j];
        if c.is_ascii_alphanumeric() || c == b'_' {
            j += 1;
        } else if c < 0x80 {
            break;
        } else {
            let ch = src[j..].chars().next().unwrap();
            if !is_xid_continue(ch) {
                break;
            }
            j += ch.len_utf8();
        }
    }
    *i = j;
    true
}

/// The canonical combining class of a code point (0 for starters).
fn ccc_of(cp: u32) -> u8 {
    if cp < 0x80 {
        return 0;
    }
    xid_table::CCC_RANGES
        .binary_search_by(|&(lo, hi, _)| {
            if cp < lo {
                std::cmp::Ordering::Greater
            } else if cp > hi {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .map_or(0, |idx| xid_table::CCC_RANGES[idx].2 as u8)
}

/// The canonical decomposition of a code point as (first, second),
/// with second 0 for a one-char mapping, or None. Hangul syllables
/// are handled by the caller, not by the table.
fn canon_decomp_of(cp: u32) -> Option<(u32, u32)> {
    if cp < 0x80 {
        return None;
    }
    xid_table::CANON_DECOMP.binary_search_by_key(&cp, |e| e.0).map_or(None, |idx| {
        let e = xid_table::CANON_DECOMP[idx];
        Some((e.1, e.2))
    })
}

const HANGUL_S_BASE: u32 = 0xAC00;
const HANGUL_L_BASE: u32 = 0x1100;
const HANGUL_V_BASE: u32 = 0x1161;
const HANGUL_T_BASE: u32 = 0x11A7;
const HANGUL_V_COUNT: u32 = 21;
const HANGUL_T_COUNT: u32 = 28;
const HANGUL_N_COUNT: u32 = HANGUL_V_COUNT * HANGUL_T_COUNT;
const HANGUL_S_COUNT: u32 = 19 * HANGUL_N_COUNT;

/// The primary composite of `a` and `b` (a starter and a following
/// char), for NFC composition: a Hangul pair, or a table pair.
fn compose_pair(a: char, b: char) -> Option<char> {
    let (x, y) = (a as u32, b as u32);
    if (HANGUL_L_BASE..HANGUL_L_BASE + 19).contains(&x) && (HANGUL_V_BASE..HANGUL_V_BASE + HANGUL_V_COUNT).contains(&y) {
        return char::from_u32(HANGUL_S_BASE + ((x - HANGUL_L_BASE) * HANGUL_V_COUNT + (y - HANGUL_V_BASE)) * HANGUL_T_COUNT);
    }
    if (HANGUL_S_BASE..HANGUL_S_BASE + HANGUL_S_COUNT).contains(&x)
        && (x - HANGUL_S_BASE) % HANGUL_T_COUNT == 0
        && (HANGUL_T_BASE + 1..HANGUL_T_BASE + HANGUL_T_COUNT).contains(&y)
    {
        return char::from_u32(x + (y - HANGUL_T_BASE));
    }
    xid_table::COMPOSE_PAIRS
        .binary_search_by(|e| (e.0, e.1).cmp(&(x, y)))
        .map_or(None, |idx| char::from_u32(xid_table::COMPOSE_PAIRS[idx].2))
}

/// Pushes the full canonical decomposition of `c` into `out`.
fn decompose_into(c: char, out: &mut Vec<char>) {
    let cp = c as u32;
    if (HANGUL_S_BASE..HANGUL_S_BASE + HANGUL_S_COUNT).contains(&cp) {
        let s = cp - HANGUL_S_BASE;
        out.push(char::from_u32(HANGUL_L_BASE + s / HANGUL_N_COUNT).unwrap());
        out.push(char::from_u32(HANGUL_V_BASE + (s % HANGUL_N_COUNT) / HANGUL_T_COUNT).unwrap());
        if s % HANGUL_T_COUNT != 0 {
            out.push(char::from_u32(HANGUL_T_BASE + s % HANGUL_T_COUNT).unwrap());
        }
        return;
    }
    match canon_decomp_of(cp) {
        Some((first, second)) => {
            decompose_into(char::from_u32(first).unwrap(), out);
            if second != 0 {
                decompose_into(char::from_u32(second).unwrap(), out);
            }
        }
        None => out.push(c),
    }
}

/// The NFC form of `s` (Unicode 17). Returns the input unchanged
/// when it is already in NFC. Used only for non-ASCII identifier
/// names, which are short, so the simple O(n^2) ordering and
/// blocked checks are fine.
fn nfc_of(s: &str) -> Cow<'_, str> {
    if s.is_ascii() {
        return Cow::Borrowed(s);
    }
    let mut decomp: Vec<char> = Vec::with_capacity(s.len());
    for ch in s.chars() {
        decompose_into(ch, &mut decomp);
    }
    // Canonical ordering: move each char left past chars with a
    // higher combining class. A starter (class 0) stops the move.
    for idx in 1..decomp.len() {
        let cc = ccc_of(decomp[idx] as u32);
        if cc == 0 {
            continue;
        }
        let mut j = idx;
        while j > 0 && ccc_of(decomp[j - 1] as u32) > cc {
            decomp.swap(j, j - 1);
            j -= 1;
        }
    }
    // Composition: compose a char into the last starter unless a
    // char between them blocks it (combining class 0, or at least
    // the composed char's class). Only chars after the last starter
    // can intervene, and none of those has class 0, so a plain
    // `>= cc` test is the blocked test; for cc 0 (a Hangul vowel
    // or trailing char) any intervening char blocks, as required.
    let mut out: Vec<char> = Vec::with_capacity(decomp.len());
    for ch in decomp {
        if let Some(si) = out.iter().rposition(|&c| ccc_of(c as u32) == 0) {
            let cc = ccc_of(ch as u32);
            let blocked = out[si + 1..].iter().any(|&c| ccc_of(c as u32) >= cc);
            if !blocked && let Some(comp) = compose_pair(out[si], ch) {
                out[si] = comp;
                continue;
            }
        }
        out.push(ch);
    }
    let normalized: String = out.into_iter().collect();
    if normalized == s { Cow::Borrowed(s) } else { Cow::Owned(normalized) }
}

/// True when `s` is already in Unicode normal form C (NFC).
fn is_nfc(s: &str) -> bool {
    matches!(nfc_of(s), Cow::Borrowed(_))
}

fn suffix(s: &str) -> Option<Option<IntSuffix>> {
    use IntSuffix::*;
    Some(Some(match s {
        "" => return Some(None),
        "i8" => I8,
        "i16" => I16,
        "i32" => I32,
        "i64" => I64,
        "isize" => Isize,
        "u8" => U8,
        "u16" => U16,
        "u32" => U32,
        "u64" => U64,
        "usize" => Usize,
        "i128" => I128,
        "u128" => U128,
        _ => return None,
    }))
}

/// Emits one doc comment as the attribute tokens it stands for:
/// `#[doc = "content"]`, or `#![doc = "content"]` for an inner comment.
/// Spans tile the comment's own bytes in token order (with one-byte gaps
/// where `stringify!` prints spaces), so positions stay inside the
/// comment and the tokens that follow keep their own.
#[allow(clippy::too_many_arguments)]
fn push_doc_tokens(
    tokens: &mut Vec<Token>,
    strings: &mut Vec<String>,
    interner: &Interner,
    file: FileId,
    base: u32,
    start: usize,
    end: usize,
    content: &str,
    inner: bool,
) {
    let at = |lo: usize, hi: usize| Span::new(file, base + lo as u32, base + hi as u32);
    let s = start;
    let shift = |n: usize| if inner { n + 1 } else { n };
    tokens.push(Token { tok: Tok::Pound, span: at(s, s + 1) });
    if inner {
        tokens.push(Token { tok: Tok::Bang, span: at(s + 1, s + 2) });
    }
    tokens.push(Token { tok: Tok::LBracket, span: at(shift(s + 1), shift(s + 2)) });
    tokens.push(Token { tok: Tok::Ident(interner.intern("doc")), span: at(shift(s + 2), shift(s + 3)) });
    tokens.push(Token { tok: Tok::Eq, span: at(shift(s + 4), shift(s + 5)) });
    let content = push_text(strings, content);
    tokens.push(Token { tok: Tok::DocStr(content), span: at(shift(s + 6).min(end), end) });
    tokens.push(Token { tok: Tok::RBracket, span: at(end, end) });
}

/// Lexes `src`, which starts at byte `base` of file `file`. Every token span names that
/// file with the root context. Stops at the first error.
/// rustc's deny-by-default lints `text_direction_codepoint_in_comment`
/// (comments) and `text_direction_codepoint_in_literal` (string and
/// char literals, doc comments): a raw codepoint that changes the visible
/// direction of text. rustc lifts them only by a crate-level
/// `#![allow]`, which lust does not read (a false reject).
fn bidi_in(src: &str, start: usize, end: usize, what: &str, file: FileId, base: u32) -> Option<Diag> {
    let text = &src[start..end];
    // Every such codepoint encodes as E2 80 AA..AE or E2 81 A6..A9.
    if !text.as_bytes().contains(&0xE2) {
        return None;
    }
    let (at, c) = text.char_indices().find(|&(_, c)| matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'))?;
    let at = (start + at) as u32;
    Some(Diag::new(
        "L0545",
        Span::new(file, base + at, base + at + c.len_utf8() as u32),
        format!("unicode codepoint changing visible direction of text present in {what}"),
        format!("Remove the codepoint, or write it as the escape `\\u{{{:X}}}` in a literal (rustc `text_direction_codepoint_in_{}`).", c as u32, if what == "comment" { "comment" } else { "literal" }),
    ))
}

pub fn lex(src: &str, file: FileId, base: u32, interner: &Interner) -> Result<Lexed, Diag> {
    let b = src.as_bytes();
    let mut tokens = Vec::with_capacity(src.len() / 4);
    let mut strings = Vec::new();
    let mut i = 0usize;
    // Local cache in front of the shared interner, so repeated names skip its lock.
    let mut cache: FxMap<&str, Sym> = FxMap::default();
    let err = |start: usize, end: usize, msg: &str| {
        Diag::new("L0001", Span::new(file, base + start as u32, base + end as u32), msg, "Fix the syntax.")
    };
    // rustc treats two names that are equal after NFC as one name.
    // Lust interns names as written, so it rejects a name that is
    // not already in NFC instead of risking two symbols for one name.
    let nfc_err = |start: usize, end: usize, name: &str| {
        Diag::new(
            "L0002",
            Span::new(file, base + start as u32, base + end as u32),
            format!("`{name}` is not in Unicode normal form C (NFC)"),
            "Rewrite the name in NFC form (use precomposed characters), or use an ASCII name.",
        )
    };
    // A shebang line (`#!/usr/bin/env ...`) at the very start of a file is
    // skipped, as in rustc. `#!` followed (after spaces) by `[` is an inner
    // attribute, not a shebang. Only a file's first bytes qualify: `base`
    // is zero exactly for a whole file, not for a chunk of one.
    if base == 0 && b.get(..2) == Some(b"#!") {
        let mut j = 2;
        while matches!(b.get(j), Some(b' ' | b'\t')) {
            j += 1;
        }
        if b.get(j) != Some(&b'[') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        }
    }
    while i < b.len() {
        let c = b[i];
        let start = i;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            // A doc comment is tokens (`#[doc = ...]`), not whitespace:
            // macro inputs count it as two token trees. `////` is plain.
            let inner = b.get(start + 2) == Some(&b'!');
            let doc = inner || (b.get(start + 2) == Some(&b'/') && b.get(start + 3) != Some(&b'/'));
            if let Some(d) = bidi_in(src, start, i, if doc { "doc comment" } else { "comment" }, file, base) {
                return Err(d);
            }
            if doc {
                let content = src[start + 3..i].strip_suffix('\r').unwrap_or(&src[start + 3..i]);
                if let Some(at) = bare_cr(content.as_bytes()) {
                    return Err(err(start + 3 + at, start + 4 + at, "bare CR not allowed in doc-comment"));
                }
                push_doc_tokens(&mut tokens, &mut strings, interner, file, base, start, i, content, inner);
            }
            continue;
        }
        if c == b'/' && b.get(i + 1) == Some(&b'*') {
            let mut depth = 0;
            while i < b.len() {
                if b[i] == b'/' && b.get(i + 1) == Some(&b'*') {
                    depth += 1;
                    i += 2;
                } else if b[i] == b'*' && b.get(i + 1) == Some(&b'/') {
                    depth -= 1;
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            if depth != 0 {
                return Err(err(start, i, "unterminated block comment"));
            }
            // `/*! .. */` and `/** .. */` are doc comments; `/***` and
            // `/**/` are plain block comments.
            let inner = b.get(start + 2) == Some(&b'!');
            let outer = b.get(start + 2) == Some(&b'*') && b.get(start + 3) != Some(&b'*') && &b[start..i] != b"/**/";
            let doc = (inner || outer) && i >= start + 5;
            if let Some(d) = bidi_in(src, start, i, if doc { "doc comment" } else { "comment" }, file, base) {
                return Err(d);
            }
            if doc {
                if let Some(at) = bare_cr(&b[start + 3..i - 2]) {
                    return Err(err(start + 3 + at, start + 4 + at, "bare CR not allowed in doc-comment"));
                }
                push_doc_tokens(&mut tokens, &mut strings, interner, file, base, start, i, &src[start + 3..i - 2], inner);
            }
            continue;
        }
        if c == b'#' && b.get(i + 1) == Some(&b'#') {
            return Err(err(start, i + 2, "reserved multi-hash token is forbidden"));
        }
        if c == b'#' && b.get(i + 1) == Some(&b'"') {
            return Err(err(start, i + 2, "invalid string literal"));
        }
        let tok = if c.is_ascii_alphabetic() || c == b'_' || (c >= 0x80 && src[i..].chars().next().is_some_and(is_xid_start)) {
            let non_ascii = scan_ident_tail(src, b, &mut i);
            let s = &src[start..i];
            // Literal prefixes: raw strings (`r".."`, `r#".."#`), byte strings and C
            // strings (`b".."`, `br".."`, `c".."`, `cr".."`) and byte chars (`b'x'`).
            // A raw string is an ordinary string with the same value, so it lexes to
            // `Tok::Str`. Byte and C forms lex to their own tokens for the parser to reject.
            if let ("r" | "br" | "cr", Some(hashes)) = (s, raw_open(b, i)) {
                let plain = s == "r";
                i += hashes + 1; // `#` run plus the opening quote
                let content = i;
                let mut close = None;
                while i < b.len() {
                    if b[i] == b'"' && (0..hashes).all(|k| b.get(i + 1 + k) == Some(&b'#')) {
                        close = Some(i);
                        break;
                    }
                    i += 1;
                }
                let Some(close) = close else {
                    return Err(err(start, i, "unterminated raw string literal"));
                };
                if let Some(at) = bare_cr(&b[content..close]) {
                    return Err(err(content + at, content + at + 1, "bare CR not allowed in raw string"));
                }
                i = close + 1 + hashes;
                if plain {
                    // The value, then the written text (never canonical: it has the `r`).
                    strings.push(src[content..close].replace("\r\n", "\n"));
                    let value = strings.len() as u32 - 1;
                    Tok::Str(value, Some(push_text(&mut strings, &src[start..i])))
                } else {
                    byte_str_tok(&mut strings, &src[start..i]).ok_or_else(|| err(start, i, "invalid raw byte or C string literal"))?
                }
            } else if (s == "b" || s == "c") && b.get(i) == Some(&b'"') {
                i += 1;
                loop {
                    match b.get(i) {
                        None => return Err(err(start, i, "unterminated byte or C string literal")),
                        Some(b'"') => {
                            i += 1;
                            break;
                        }
                        Some(b'\\') => i = (i + 2).min(b.len()),
                        Some(b'\r') if b.get(i + 1) != Some(&b'\n') => {
                            return Err(err(i, i + 1, "bare CR not allowed in string, use `\\r` instead"));
                        }
                        Some(_) => i += 1,
                    }
                }
                byte_str_tok(&mut strings, &src[start..i]).ok_or_else(|| err(start, i, "invalid byte or C string literal"))?
            } else if s == "b" && b.get(i) == Some(&b'\'') {
                i += 1;
                if b.get(i) == Some(&b'\\') {
                    i += 2;
                    if b.get(i.wrapping_sub(1)) == Some(&b'x') {
                        i = (i + 2).min(b.len());
                    }
                } else if let Some(ch) = src[i..].chars().next() {
                    if let Some(esc) = must_escape(ch) {
                        return Err(err(i, i + ch.len_utf8(), &format!("byte constant must be escaped: `{esc}`")));
                    }
                    i += ch.len_utf8();
                }
                if b.get(i) != Some(&b'\'') {
                    return Err(err(start, i, "unterminated byte char literal"));
                }
                i += 1;
                if decode_byte_char(&src[start..i]).is_none() {
                    return Err(err(start, i, "invalid byte char literal"));
                }
                strings.push(src[start..i].to_string());
                Tok::ByteChar(strings.len() as u32 - 1)
            } else if raw_open(b, i).is_some() || b.get(i) == Some(&b'\'') {
                // An identifier right against a quote is a reserved
                // literal prefix (`f"x"`, `f#"x"#`, `f'x'`): rustc
                // rejects every prefix but the ones handled above.
                return Err(Diag::new(
                    "L0001",
                    Span::new(file, base + start as u32, base + i as u32),
                    format!("prefix `{s}` is unknown"),
                    "Prefixed identifiers and literals are reserved. Insert whitespace, or use a known prefix (r, b, c, br, cr).",
                ));
            }
            // A raw identifier: `r#name`. The name may be any keyword except a few (see above).
            else if s == "r" && b.get(i) == Some(&b'#') && is_ident_start_at(src, b, i + 1) {
                i += 1;
                let name_start = i;
                let name_non_ascii = scan_ident_tail(src, b, &mut i);
                let name = &src[name_start..i];
                if raw_ident_invalid(name) {
                    return Err(Diag::new(
                        "L0001",
                        Span::new(file, base + start as u32, base + i as u32),
                        format!("`r#{name}` is not a valid identifier"),
                        "Use a different name. `self`, `Self`, `super`, `crate`, and `_` cannot be raw identifiers.",
                    ));
                }
                if name_non_ascii && !is_nfc(name) {
                    return Err(nfc_err(start, i, name));
                }
                Tok::Ident(*cache.entry(name).or_insert_with(|| interner.intern(name)))
            } else if let Some(k) = keyword(s) {
                k
            } else if reserved(s) {
                Tok::Reserved(*cache.entry(s).or_insert_with(|| interner.intern(s)))
            } else {
                if non_ascii && !is_nfc(s) {
                    return Err(nfc_err(start, i, s));
                }
                Tok::Ident(*cache.entry(s).or_insert_with(|| interner.intern(s)))
            }
        } else if c.is_ascii_digit() {
            lex_number(src, &mut i, &mut strings).ok_or_else(|| err(start, i, "invalid number literal"))?
        } else if c == b'"' {
            i += 1;
            let mut s = String::new();
            // The text is canonical (`lit_text` prints it back from the
            // value) with no escape and only printable ASCII but `'`.
            let mut canonical = true;
            loop {
                match b.get(i) {
                    None => return Err(err(start, i, "unterminated string literal")),
                    Some(b'"') => {
                        i += 1;
                        break;
                    }
                    Some(b'\\') => {
                        canonical = false;
                        // A backslash before a newline skips the newline and
                        // the spaces, tabs, and newlines after it.
                        let newline = match (b.get(i + 1), b.get(i + 2)) {
                            (Some(b'\n'), _) => 1,
                            (Some(b'\r'), Some(b'\n')) => 2,
                            _ => 0,
                        };
                        if newline > 0 {
                            i += 1 + newline;
                            while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') {
                                i += 1;
                            }
                            continue;
                        }
                        let ch = lex_escape(src, &mut i).ok_or_else(|| err(i, i + 2, "invalid escape"))?;
                        s.push(ch);
                    }
                    // rustc reads CRLF as LF (the LF is read next round); a bare CR is an error.
                    Some(b'\r') => {
                        if b.get(i + 1) != Some(&b'\n') {
                            return Err(err(i, i + 1, "bare CR not allowed in string, use `\\r` instead"));
                        }
                        canonical = false;
                        i += 1;
                    }
                    Some(_) => {
                        let ch = src[i..].chars().next().unwrap();
                        canonical &= (' '..='~').contains(&ch) && ch != '\'';
                        s.push(ch);
                        i += ch.len_utf8();
                    }
                }
            }
            strings.push(s);
            let value = strings.len() as u32 - 1;
            Tok::Str(value, (!canonical).then(|| push_text(&mut strings, &src[start..i])))
        } else if c == b'\'' {
            // Either a char literal or a lifetime.
            let next = src[i + 1..].chars().next();
            let after = next.map(|ch| i + 1 + ch.len_utf8());
            if next.is_some_and(|ch| ch == '_' || is_xid_start(ch)) && after.and_then(|a| b.get(a)) != Some(&b'\'') {
                i += 1;
                let mut name_start = i;
                let mut name_non_ascii = scan_ident_tail(src, b, &mut i);
                // A raw lifetime: `'r#name`. The token keeps the name; its
                // span still covers the `r#`, which macro matching reads.
                if &src[name_start..i] == "r" && b.get(i) == Some(&b'#') && is_ident_start_at(src, b, i + 1) {
                    i += 1;
                    name_start = i;
                    name_non_ascii = scan_ident_tail(src, b, &mut i);
                    let name = &src[name_start..i];
                    if raw_ident_invalid(name) {
                        return Err(Diag::new(
                            "L0001",
                            Span::new(file, base + start as u32, base + i as u32),
                            format!("`{name}` cannot be a raw lifetime"),
                            "Use a different name. `self`, `Self`, `super`, `crate`, and `_` cannot be raw lifetimes.",
                        ));
                    }
                }
                let name = &src[name_start..i];
                if name_non_ascii && !is_nfc(name) {
                    return Err(nfc_err(start, i, name));
                }
                Tok::Lifetime(*cache.entry(name).or_insert_with(|| interner.intern(name)))
            } else {
                i += 1;
                let escaped = b.get(i) == Some(&b'\\');
                let ch = if escaped {
                    lex_escape(src, &mut i).ok_or_else(|| err(i, i + 2, "invalid escape"))?
                } else {
                    let ch = src[i..].chars().next().ok_or_else(|| err(start, i, "bad char literal"))?;
                    if let Some(esc) = must_escape(ch) {
                        return Err(err(i, i + ch.len_utf8(), &format!("character constant must be escaped: `{esc}`")));
                    }
                    i += ch.len_utf8();
                    ch
                };
                if b.get(i) != Some(&b'\'') {
                    return Err(err(start, i, "unterminated char literal"));
                }
                i += 1;
                // Canonical text (see the string case): unescaped printable ASCII.
                let canonical = !escaped && (' '..='~').contains(&ch);
                Tok::Char(ch, (!canonical).then(|| push_text(&mut strings, &src[start..i])))
            }
        } else {
            // `#` runs are reserved in edition 2024: a guarded string
            // (`#"x"#`) and a multi-hash (`##`) are lexer errors, not
            // a `#` token next to a literal or another `#`.
            if c == b'#' && b.get(i + 1) == Some(&b'"') {
                return Err(err(start, start + 2, "invalid string literal: unprefixed guarded strings are reserved"));
            }
            if c == b'#' && b.get(i + 1) == Some(&b'#') {
                return Err(err(start, start + 2, "reserved multi-hash token is forbidden"));
            }
            let two = if i + 1 < b.len() { &b[i..i + 2] } else { &b[i..i + 1] };
            let three = if i + 2 < b.len() { &b[i..i + 3] } else { &[][..] };
            let (t, n) = if three == b"..=" {
                (Tok::DotDotEq, 3)
            } else if three == b"..." {
                (Tok::Ellipsis, 3)
            } else if three == b"<<=" {
                (Tok::ShlEq, 3)
            } else {
                match two {
                    b"::" => (Tok::PathSep, 2),
                    b".." => (Tok::DotDot, 2),
                    b"->" => (Tok::Arrow, 2),
                    b"=>" => (Tok::FatArrow, 2),
                    b"==" => (Tok::EqEq, 2),
                    b"!=" => (Tok::Ne, 2),
                    b"<=" => (Tok::Le, 2),
                    b">=" => (Tok::Ge, 2),
                    b"+=" => (Tok::PlusEq, 2),
                    b"-=" => (Tok::MinusEq, 2),
                    b"*=" => (Tok::StarEq, 2),
                    b"/=" => (Tok::SlashEq, 2),
                    b"%=" => (Tok::PercentEq, 2),
                    b"&=" => (Tok::AmpEq, 2),
                    b"|=" => (Tok::PipeEq, 2),
                    b"^=" => (Tok::CaretEq, 2),
                    b"&&" => (Tok::AndAnd, 2),
                    b"||" => (Tok::OrOr, 2),
                    b"<<" => (Tok::Shl, 2),
                    _ => (
                        match c {
                            b'(' => Tok::LParen,
                            b')' => Tok::RParen,
                            b'{' => Tok::LBrace,
                            b'}' => Tok::RBrace,
                            b'[' => Tok::LBracket,
                            b']' => Tok::RBracket,
                            b',' => Tok::Comma,
                            b';' => Tok::Semi,
                            b':' => Tok::Colon,
                            b'.' => Tok::Dot,
                            b'=' => Tok::Eq,
                            b'<' => Tok::Lt,
                            b'>' => Tok::Gt,
                            b'+' => Tok::Plus,
                            b'-' => Tok::Minus,
                            b'*' => Tok::Star,
                            b'/' => Tok::Slash,
                            b'%' => Tok::Percent,
                            b'&' => Tok::Amp,
                            b'|' => Tok::Pipe,
                            b'^' => Tok::Caret,
                            b'!' => Tok::Bang,
                            b'#' => Tok::Pound,
                            b'?' => Tok::Question,
                            b'$' => Tok::Dollar,
                            b'@' => Tok::At,
                            _ => {
                                return Err(err(
                                    start,
                                    start + src[start..].chars().next().map_or(1, char::len_utf8),
                                    "unexpected character",
                                ));
                            }
                        },
                        1,
                    ),
                }
            };
            i += n;
            t
        };
        if matches!(tok, Tok::Str(..) | Tok::Char(..))
            && let Some(d) = bidi_in(src, start, i, "literal", file, base)
        {
            return Err(d);
        }
        tokens.push(Token { tok, span: Span::new(file, base + start as u32, base + i as u32) });
    }
    let end = base + b.len() as u32;
    tokens.push(Token { tok: Tok::Eof, span: Span::new(file, end, end) });
    Ok(Lexed { tokens, strings })
}

// `>>` is never lexed as one token, so generics like `Vec<Vec<i64>>` parse without splitting.
// The parser joins two `>` tokens into a shift when they touch.

/// Stores a literal's written text and returns its `Lexed::strings`
/// index. rustc reads the file with every CRLF as LF, so the text does.
fn push_text(strings: &mut Vec<String>, text: &str) -> u32 {
    strings.push(if text.contains('\r') { text.replace("\r\n", "\n") } else { text.to_string() });
    strings.len() as u32 - 1
}

/// The offset of the first CR in `bytes` that does not start a CRLF:
/// rustc rejects one inside a string literal.
fn bare_cr(bytes: &[u8]) -> Option<usize> {
    (0..bytes.len()).find(|&k| bytes[k] == b'\r' && bytes.get(k + 1) != Some(&b'\n'))
}

/// How rustc spells a char that a char or byte literal must escape, or
/// None when it may stand as written.
fn must_escape(ch: char) -> Option<&'static str> {
    Some(match ch {
        '\'' => "'",
        '\n' => "\\n",
        '\r' => "\\r",
        '\t' => "\\t",
        _ => return None,
    })
}

/// One byte escape at `b[i]` (the byte after the backslash): the byte
/// and the index just past the escape. Byte escapes take the full
/// `\xHH` range; `\u` is not a byte escape.
fn byte_escape(b: &[u8], i: usize) -> Option<(u8, usize)> {
    let c = *b.get(i)?;
    Some(match c {
        b'n' => (b'\n', i + 1),
        b't' => (b'\t', i + 1),
        b'r' => (b'\r', i + 1),
        b'0' => (0, i + 1),
        b'\\' => (b'\\', i + 1),
        b'\'' => (b'\'', i + 1),
        b'"' => (b'"', i + 1),
        b'x' => {
            let h = |c: u8| (c as char).to_digit(16).map(|v| v as u8);
            let v = h(*b.get(i + 1)?)? * 16 + h(*b.get(i + 2)?)?;
            (v, i + 3)
        }
        _ => return None,
    })
}

/// The bytes of a byte char literal's verbatim token text (`b'x'`),
/// when it holds exactly one ASCII byte or byte escape.
pub fn decode_byte_char(text: &str) -> Option<u8> {
    let inner = text.strip_prefix("b'")?.strip_suffix('\'')?;
    let b = inner.as_bytes();
    match b.first() {
        Some(b'\\') => {
            let (v, next) = byte_escape(b, 1)?;
            if next == b.len() { Some(v) } else { None }
        }
        Some(&c) if b.len() == 1 && c.is_ascii() && c != b'\'' => Some(c),
        _ => None,
    }
}

/// The bytes of a byte string literal's verbatim token text (`b".."`
/// or a raw `br".."`, `br#".."#`). Non-ASCII text is not a byte
/// string in Rust, so it rejects here like any bad escape.
pub fn decode_byte_str(text: &str) -> Option<Vec<u8>> {
    if let Some(rest) = text.strip_prefix("br") {
        let hashes = rest.bytes().take_while(|&c| c == b'#').count();
        let close = format!("\"{}", "#".repeat(hashes));
        let inner = rest[hashes..].strip_prefix('"')?.strip_suffix(&close)?;
        if inner.bytes().any(|c| !c.is_ascii()) {
            return None;
        }
        return Some(inner.bytes().collect());
    }
    let inner = text.strip_prefix("b\"")?.strip_suffix('"')?;
    let b = inner.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'\\' => {
                if b.get(i + 1) == Some(&b'\n') {
                    i += 2;
                    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') {
                        i += 1;
                    }
                    continue;
                }
                let (v, next) = byte_escape(b, i + 1)?;
                out.push(v);
                i = next;
            }
            c if c.is_ascii() => {
                out.push(c);
                i += 1;
            }
            _ => return None,
        }
    }
    Some(out)
}

/// Stores a byte or C string literal's text (`b".."`, `br".."`, `c".."`,
/// `cr".."`) and returns its token, or None when rustc's lexer rejects
/// the literal (a bad escape, a non-ASCII byte, a NUL in a C string),
/// as it does in macro input too.
fn byte_str_tok(strings: &mut Vec<String>, text: &str) -> Option<Tok> {
    let i = push_text(strings, text);
    let t = &strings[i as usize];
    let ok = if t.starts_with('c') { c_str_ok(t) } else { decode_byte_str(t).is_some() };
    ok.then_some(Tok::ByteStr(i))
}

/// True when a C string literal's text (`c".."`, `cr#".."#`) is one rustc
/// accepts: known escapes (bytes up to `\xFF`, `\u{..}`) and no NUL.
fn c_str_ok(text: &str) -> bool {
    if text.starts_with("cr") {
        return !text.contains('\0');
    }
    let Some(inner) = text.strip_prefix("c\"").and_then(|t| t.strip_suffix('"')) else { return false };
    let b = inner.as_bytes();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            0 => return false,
            b'\\' => match b.get(i + 1) {
                Some(b'\n') => {
                    i += 2;
                    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') {
                        i += 1;
                    }
                }
                Some(b'x') => match inner.get(i + 2..i + 4).and_then(|h| u8::from_str_radix(h, 16).ok()) {
                    Some(v) if v != 0 && inner.as_bytes()[i + 2].is_ascii_hexdigit() => i += 4,
                    _ => return false,
                },
                _ => match lex_escape(inner, &mut i) {
                    Some(c) if c != '\0' => {}
                    _ => return false,
                },
            },
            _ => i += 1,
        }
    }
    true
}

/// If a raw string opening (`#` run plus `"`) starts at `i`, returns the `#` count.
fn raw_open(b: &[u8], i: usize) -> Option<usize> {
    let mut hashes = 0;
    while b.get(i + hashes) == Some(&b'#') {
        hashes += 1;
    }
    (b.get(i + hashes) == Some(&b'"')).then_some(hashes)
}

/// Stores a float literal's text in `strings` and returns its token.
fn float_tok(src: &str, start: usize, end: usize, strings: &mut Vec<String>) -> Tok {
    strings.push(src[start..end].to_string());
    Tok::Float(strings.len() as u32 - 1)
}

fn lex_number(src: &str, i: &mut usize, strings: &mut Vec<String>) -> Option<Tok> {
    let b = src.as_bytes();
    let start = *i;
    let (radix, digits_start) = match (b[start], b.get(start + 1)) {
        (b'0', Some(b'x')) => (16, start + 2),
        (b'0', Some(b'o')) => (8, start + 2),
        (b'0', Some(b'b')) => (2, start + 2),
        _ => (10, start),
    };
    *i = digits_start;
    while *i < b.len() && (b[*i].is_ascii_hexdigit() && radix == 16 || b[*i].is_ascii_digit() || b[*i] == b'_') {
        *i += 1;
    }
    // A float literal: `1.5`, `1.`, or `1e3`. Not `1..2` (range) and
    // not `1.foo()` (field): the dot joins the float unless another
    // dot or an identifier start follows it.
    let dot_joins = radix == 10 && b.get(*i) == Some(&b'.') && b.get(*i + 1) != Some(&b'.') && !is_ident_start_at(src, b, *i + 1);
    if dot_joins {
        *i += 1;
        while *i < b.len() && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_') {
            *i += 1;
        }
        // An exponent sign right after `e` belongs to the literal (`1.5e-3`).
        if *i >= 2 && (b[*i - 1] == b'e' || b[*i - 1] == b'E') && matches!(b.get(*i), Some(b'+' | b'-')) {
            *i += 1;
            while *i < b.len() && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_') {
                *i += 1;
            }
        }
        return Some(float_tok(src, start, *i, strings));
    }
    // A trailing dot with no digits after it is still a float (`1.`) when
    // the dot does not start a range (`1..2`) or a field or method name
    // (`1.foo()`): rustc lexes it as one float literal token.
    if radix == 10 && b.get(*i) == Some(&b'.') && b.get(*i + 1) != Some(&b'.') && !is_ident_start_at(src, b, *i + 1) {
        *i += 1;
        return Some(float_tok(src, start, *i, strings));
    }
    // An exponent with no dot: `1e3`, `1e-3`, `1e3f64`. Not an identifier suffix:
    // the exponent needs at least one digit after the optional sign.
    if radix == 10 && matches!(b.get(*i), Some(b'e' | b'E')) {
        let mut j = *i + 1;
        if matches!(b.get(j), Some(b'+' | b'-')) {
            j += 1;
        }
        if b.get(j).is_some_and(|c| c.is_ascii_digit()) {
            *i = j;
            while *i < b.len() && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_') {
                *i += 1;
            }
            return Some(float_tok(src, start, *i, strings));
        }
    }
    let digits_end = *i;
    let suf_start = *i;
    while *i < b.len() && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_') {
        *i += 1;
    }
    let suf = &src[suf_start..*i];
    if suf == "f32" || suf == "f64" {
        return Some(float_tok(src, start, *i, strings));
    }
    // Accumulate digits directly; underscores are separators.
    let mut value: u128 = 0;
    let mut any = false;
    for &c in &b[digits_start..digits_end] {
        if c == b'_' {
            continue;
        }
        let d = (c as char).to_digit(radix)? as u128;
        value = value.checked_mul(radix as u128)?.checked_add(d)?;
        any = true;
    }
    if !any {
        return None;
    }
    let suffix = suffix(suf)?;
    // The text is canonical (`lit_text` prints it back from the value)
    // when it is plain decimal: no radix prefix, `_`, or leading zero.
    let text = &src[start..*i];
    let canonical = radix == 10 && !text.contains('_') && !(text.len() > 1 && text.starts_with('0') && b[start + 1].is_ascii_digit());
    Some(Tok::Int(value, suffix, (!canonical).then(|| push_text(strings, text))))
}

fn lex_escape(src: &str, i: &mut usize) -> Option<char> {
    let b = src.as_bytes();
    let c = *b.get(*i + 1)?;
    *i += 2;
    Some(match c {
        b'n' => '\n',
        b't' => '\t',
        b'r' => '\r',
        b'0' => '\0',
        b'\\' => '\\',
        b'\'' => '\'',
        b'"' => '"',
        // Exactly two hex digits (`from_str_radix` alone would take `+1`).
        b'x' => {
            let digits = src.get(*i..*i + 2).filter(|d| d.bytes().all(|c| c.is_ascii_hexdigit()))?;
            let v = u8::from_str_radix(digits, 16).ok()?;
            *i += 2;
            if v > 0x7f {
                return None;
            }
            v as char
        }
        // `\u{..}`: a hex digit first, then hex digits and `_`, at most six digits.
        b'u' => {
            if b.get(*i) != Some(&b'{') {
                return None;
            }
            let end = src[*i..].find('}')? + *i;
            let digits = &src[*i + 1..end];
            let hex = digits.bytes().filter(u8::is_ascii_hexdigit).count();
            if !digits.starts_with(|c: char| c.is_ascii_hexdigit()) || hex > 6 || hex + digits.matches('_').count() != digits.len() {
                return None;
            }
            let v = u32::from_str_radix(&digits.replace('_', ""), 16).ok()?;
            *i = end + 1;
            char::from_u32(v)?
        }
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A packed stream unpacks to the same tokens and strings, spans that jump back (a macro
    /// call's tokens) and another file and context included.
    #[test]
    fn a_packed_stream_unpacks_token_for_token() {
        let interner = Interner::default();
        let mut l = lex("fn f(x: u32) -> u32 { let s = \"a\"; x + 1_000u32 }\n/// doc\nstruct S;", FileId(3), 40, &interner).unwrap();
        l.tokens.push(Token { tok: Tok::Comma, span: Span { file: FileId(9), lo: 2, hi: 3, ctxt: crate::diag::CtxtId(5) } });
        l.tokens.push(Token { tok: l.tokens[0].tok, span: Span { file: FileId(3), lo: 41, hi: 44, ctxt: crate::diag::CtxtId(0) } });
        let back = PackedLexed::pack(&l).unpack();
        assert_eq!(back.strings, l.strings);
        assert_eq!(back.tokens.len(), l.tokens.len());
        for (a, b) in back.tokens.iter().zip(&l.tokens) {
            assert_eq!((a.tok, a.span), (b.tok, b.span));
        }
    }

    /// XID properties at known chars, as rustc 1.99 (Unicode 17)
    /// decides them: emoji and circled letters are not identifier
    /// chars, a combining mark continues but does not start, and a
    /// char that Unicode 18 added to XID_Start does not start.
    #[test]
    fn xid_properties() {
        assert!(is_xid_start('a') && is_xid_continue('a'));
        assert!(!is_xid_start('_') && is_xid_continue('_'));
        assert!(is_xid_start('α') && is_xid_continue('α'));
        assert!(is_xid_start('变') && is_xid_continue('变'));
        assert!(!is_xid_start('\u{301}') && is_xid_continue('\u{301}'));
        assert!(!is_xid_start('🦀') && !is_xid_continue('🦀'));
        assert!(!is_xid_start('\u{24b6}') && !is_xid_continue('\u{24b6}'));
        assert!(!is_xid_start('\u{37a}') && !is_xid_continue('\u{37a}'));
        assert!(!is_xid_start('\u{ff3f}') && is_xid_continue('\u{ff3f}'));
        assert!(!is_xid_start('\u{558}') && !is_xid_continue('\u{558}'));
        // Every XID_Start char is also XID_Continue; the lexer's
        // non-ASCII scan relies on it for the first char.
        for &(lo, hi) in xid_table::XID_START_RANGES {
            for cp in [lo, hi] {
                assert!(in_ranges(xid_table::XID_CONTINUE_RANGES, cp), "U+{cp:04X}");
            }
        }
    }

    /// NFC checks at the forms the lexer must tell apart, as
    /// decided against rustc 1.99: precomposed text and a
    /// combining mark that cannot compose are NFC; a decomposed
    /// letter, out-of-order marks, and Hangul jamo are not.
    #[test]
    fn nfc_forms() {
        assert!(is_nfc("abc"));
        assert!(is_nfc("caf\u{e9}"));
        assert!(is_nfc("x\u{301}"));
        assert!(is_nfc("αβγ"));
        assert!(is_nfc("\u{ac00}"));
        assert!(!is_nfc("cafe\u{301}"));
        assert!(!is_nfc("a\u{315}\u{300}"));
        assert!(!is_nfc("\u{1100}\u{1161}"));
        assert_eq!(nfc_of("cafe\u{301}"), "caf\u{e9}");
        assert_eq!(nfc_of("a\u{315}\u{300}"), "\u{e0}\u{315}");
        assert_eq!(nfc_of("\u{1100}\u{1161}"), "\u{ac00}");
    }
}
