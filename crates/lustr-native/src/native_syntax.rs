//! Grammar composition for Lustr's unsafe function contracts. The application
//! text is never rewritten. This parser consumes the additional grammar tokens,
//! delegates the common grammar to Lust, and retains every consumed qualifier
//! in a per-program semantic record before the checked backend can see it.

use crate::ast::{Ast, Expr, ItemKind, TyExpr};
use crate::diag::{Diag, Span};
use crate::intern::{Interner, Known};
use crate::lexer::{Lexed, Tok, Token};
use crate::native_safety::NativeSafety;

pub struct Parsed {
    pub ast: Ast,
    pub safety: NativeSafety,
}

fn unsupported(span: Span, message: &str) -> Diag {
    Diag::new("L1000", span, message, "This native unsafe interface is not implemented yet.")
}

#[derive(Clone, Copy)]
enum Marker {
    Function { name: Span, qualifier: Span },
    Block { open: Span, qualifier: Span },
    Pointer { function: Span, qualifier: Span },
}

fn close(tok: Tok) -> Option<Tok> {
    match tok {
        Tok::LParen => Some(Tok::RParen),
        Tok::LBrace => Some(Tok::RBrace),
        Tok::LBracket => Some(Tok::RBracket),
        _ => None,
    }
}

fn group_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut stack = vec![close(tokens.get(start)?.tok)?];
    for (i, token) in tokens.iter().enumerate().skip(start + 1) {
        if let Some(end) = close(token.tok) {
            stack.push(end);
        } else if matches!(token.tok, Tok::RParen | Tok::RBrace | Tok::RBracket) {
            if stack.pop() != Some(token.tok) {
                return None;
            }
            if stack.is_empty() {
                return Some(i + 1);
            }
        }
    }
    None
}

/// Leave an unchanged token stream borrowed. Only recognized extension
/// qualifiers require an owned stream, copied in contiguous ranges between
/// qualifiers. The pinned parser still owns all common grammar and every span.
pub fn parse(lexed: &Lexed, interner: &Interner, known: &Known) -> Result<Parsed, Diag> {
    let mut markers = Vec::new();
    let mut tokens: Option<Vec<Token>> = None;
    let mut copied = 0;
    let mut i = 0;
    while i < lexed.tokens.len() {
        let token = lexed.tokens[i];
        // A macro definition is an opaque token grammar owned by the existing
        // expander. Its unexpanded branches are not unsafe executable contexts.
        if matches!(token.tok, Tok::Ident(s) if interner.get(s) == "macro_rules")
            && lexed.tokens.get(i + 1).is_some_and(|t| t.tok == Tok::Bang)
            && lexed.tokens.get(i + 2).is_some_and(|t| matches!(t.tok, Tok::Ident(_)))
            && lexed.tokens.get(i + 3).is_some_and(|t| close(t.tok).is_some())
        {
            i = group_end(&lexed.tokens, i + 3).ok_or_else(|| unsupported(token.span, "unbalanced macro definition"))?;
            continue;
        }
        if token.tok != Tok::Unsafe {
            i += 1;
            continue;
        }
        let next = lexed.tokens.get(i + 1).copied();
        let marker = match next.map(|t| t.tok) {
            Some(Tok::Fn) => match lexed.tokens.get(i + 2).copied() {
                Some(Token { tok: Tok::Ident(_), span }) => Marker::Function { name: span, qualifier: token.span },
                Some(Token { tok: Tok::LParen, .. }) => Marker::Pointer { function: next.unwrap().span, qualifier: token.span },
                _ => return Err(unsupported(token.span, "expected a function declaration or pointer type after `unsafe fn`")),
            },
            Some(Tok::LBrace) => {
                if i > 0 && lexed.tokens[i - 1].tok == Tok::Else {
                    return Err(unsupported(token.span, "an else branch requires a block or if expression before an unsafe expression"));
                }
                Marker::Block { open: next.unwrap().span, qualifier: token.span }
            }
            // These are already parsed and checked by the pinned implementation.
            Some(Tok::Impl | Tok::Trait) => {
                i += 1;
                continue;
            }
            _ => return Err(unsupported(token.span, "this unsafe declaration or operation is not implemented")),
        };
        markers.push(marker);
        tokens.get_or_insert_with(|| Vec::with_capacity(lexed.tokens.len() - 1))
            .extend_from_slice(&lexed.tokens[copied..i]);
        copied = i + 1;
        i += 1;
    }
    let rewritten = tokens.map(|mut tokens| {
        tokens.extend_from_slice(&lexed.tokens[copied..]);
        Lexed { tokens, strings: lexed.strings.clone() }
    });
    let Some(rewritten) = rewritten else {
        return crate::parser::parse(lexed, interner, known).map(|ast| Parsed { ast, safety: NativeSafety::default() });
    };
    let mut ast = crate::parser::parse(&rewritten, interner, known)?;
    let mut safety = NativeSafety::default();
    for marker in markers {
        match marker {
            Marker::Function { name, qualifier } => {
                let matches: Vec<usize> = ast
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(i, item)| matches!(&item.kind, ItemKind::Fn(def) if def.name_span == name).then_some(i))
                    .collect();
                if matches.len() != 1 {
                    return Err(unsupported(qualifier, "unsafe functions currently require a unique crate-root declaration"));
                }
                let item = &mut ast.items[matches[0]];
                let ItemKind::Fn(def) = &mut item.kind else { unreachable!() };
                if def.qual.is_const || def.qual.is_async || def.qual.abi.is_some() || def.qual.is_unsafe {
                    return Err(unsupported(qualifier, "combined unsafe function qualifiers are not implemented"));
                }
                def.qual.is_unsafe = true;
                safety.function_qualifiers.push((name, qualifier));
            }
            Marker::Block { open, qualifier } => {
                let matches: Vec<u32> = ast
                    .blocks
                    .iter()
                    .enumerate()
                    .filter_map(|(i, block)| {
                        (block.span.file == open.file && block.span.ctxt == open.ctxt && block.span.lo == open.lo).then_some(i as u32)
                    })
                    .collect();
                if matches.len() != 1 {
                    return Err(unsupported(qualifier, "unsafe block syntax could not be bound to one native AST block"));
                }
                if !ast.exprs.iter().any(|expr| matches!(expr, Expr::Block(block) if block.0 == matches[0])) {
                    return Err(unsupported(
                        qualifier,
                        "an unsafe expression cannot replace a required function, loop or conditional body block",
                    ));
                }
                if safety.unsafe_blocks.insert(matches[0], qualifier).is_some() {
                    return Err(unsupported(qualifier, "duplicate unsafe block qualifier"));
                }
            }
            Marker::Pointer { function, qualifier } => {
                let mut matches: Vec<(u32, u32)> = ast
                    .tys
                    .iter()
                    .enumerate()
                    .filter_map(|(i, ty)| {
                        let span = ast.ty_spans[i];
                        (matches!(ty, TyExpr::FnPtr { .. })
                            && span.file == function.file
                            && span.ctxt == function.ctxt
                            && span.lo <= function.lo
                            && function.hi <= span.hi)
                            .then_some((span.hi - span.lo, i as u32))
                    })
                    .collect();
                matches.sort_unstable();
                if matches.is_empty() || (matches.len() > 1 && matches[0].0 == matches[1].0) {
                    return Err(unsupported(qualifier, "unsafe pointer syntax could not be bound to one native AST type"));
                }
                if safety.unsafe_pointer_types.insert(matches[0].1, qualifier).is_some() {
                    return Err(unsupported(qualifier, "duplicate unsafe function-pointer qualifier"));
                }
            }
        }
    }
    safety.validate_ast(&ast, interner)?;
    Ok(Parsed { ast, safety })
}
