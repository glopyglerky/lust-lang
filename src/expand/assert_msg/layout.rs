// Line layout for the long expression forms supported by assert_msg.
// The boxes and breaks follow rustc_ast_pretty 1.99: each expression has
// a four-space inconsistent box, argument lists have a zero-offset box,
// and record fields break consistently. The finite token stream lets us
// measure boxes and break sections before printing, without a ring buffer.
// This path runs only after the existing flat printer exceeds its margin.
use super::{Delim, K, MARGIN, Op, P, Prec, Tok};

enum Doc {
    Word(String),
    Break { spaces: usize, offset: isize, comma: bool },
    Group { indent: isize, consistent: bool, depth: usize, parts: Vec<Doc> },
    Limit,
}

const MAX_TOKENS: usize = 512;
const MAX_TEXT: usize = 64 * 1024;
const MAX_DEPTH: usize = 128;
const MAX_OUTPUT: usize = 64 * 1024;

fn depth(doc: &Doc) -> usize {
    match doc {
        Doc::Group { depth, .. } => *depth,
        Doc::Limit => MAX_DEPTH + 1,
        _ => 0,
    }
}

fn bounded(doc: Doc) -> Option<Doc> {
    (depth(&doc) <= MAX_DEPTH).then_some(doc)
}

fn word(text: impl Into<String>) -> Doc {
    Doc::Word(text.into())
}

fn space() -> Doc {
    Doc::Break { spaces: 1, offset: 0, comma: false }
}

fn group(indent: isize, consistent: bool, parts: Vec<Doc>) -> Doc {
    let depth = parts.iter().map(depth).max().unwrap_or(0) + 1;
    // Reject before creating an overdeep parent. Every child dropped on
    // this path already satisfies the same depth bound.
    if depth > MAX_DEPTH {
        Doc::Limit
    } else {
        Doc::Group { indent, consistent, depth, parts }
    }
}

fn expr(parts: Vec<Doc>) -> Doc {
    group(4, false, parts)
}

fn comma_list(items: Vec<Doc>) -> Doc {
    let mut parts = Vec::new();
    for item in items {
        if !parts.is_empty() {
            parts.push(word(","));
            parts.push(space());
        }
        parts.push(item);
    }
    group(0, false, parts)
}

pub(super) fn print(parser: &mut P, expected_flat: &str) -> Option<String> {
    // Expansion can ask for an assertion's text before the main parser
    // checks expression-chain limits. These cold-path budgets bound the
    // second parse, recursive document walks/drop, and rendered padding.
    if parser.t.len() > MAX_TOKENS || expected_flat.len() > MAX_TEXT {
        return None;
    }
    let doc = parse_expr(parser, Prec::Jump)?;
    if parser.i != parser.t.len() {
        return None;
    }
    let mut flat = String::new();
    flatten(&doc, &mut flat);
    // The full flat parser owns precedence, fragment and grammar fixups.
    // A layout form is admitted only when it describes the same expression.
    if flat != expected_flat {
        return None;
    }
    render(doc)
}

fn flatten(doc: &Doc, out: &mut String) {
    match doc {
        Doc::Word(text) => out.push_str(text),
        Doc::Break { spaces, .. } => out.extend(std::iter::repeat_n(' ', *spaces)),
        Doc::Group { parts, .. } => {
            for part in parts {
                flatten(part, out);
            }
        }
        Doc::Limit => unreachable!("bounded layout document"),
    }
}

fn parse_expr(p: &mut P, minimum: Prec) -> Option<Doc> {
    let mut left = prefix(p)?;
    while let Some((op, count)) = p.binop() {
        let Op::Bin(operator, precedence) = op else { return None };
        if precedence < minimum {
            break;
        }
        p.i += count;
        let right = parse_expr(p, precedence.next())?;
        left = bounded(expr(vec![left, space(), word(operator), space(), right]))?;
    }
    Some(left)
}

fn prefix(p: &mut P) -> Option<Doc> {
    let operator = match p.tok() {
        Some(Tok::Minus) => "-",
        Some(Tok::Bang) => "!",
        Some(Tok::Star) => "*",
        Some(Tok::Amp | Tok::AndAnd) => {
            let double = p.is(Tok::AndAnd);
            p.i += 1;
            let mutable = p.eat(Tok::Mut);
            let value = prefix(p)?;
            let reference = expr(vec![word(if mutable { "&mut " } else { "&" }), value]);
            return bounded(if double { expr(vec![word("&"), reference]) } else { reference });
        }
        _ => return postfix(p),
    };
    p.i += 1;
    bounded(expr(vec![word(operator), prefix(p)?]))
}

fn postfix(p: &mut P) -> Option<Doc> {
    let mut value = bounded(primary(p)?)?;
    loop {
        if p.open(Delim::Paren) {
            let args = arguments(p)?;
            value = expr(vec![value, word("("), comma_list(args), word(")")]);
        } else if p.eat(Tok::Dot) {
            let named = matches!(p.tok(), Some(Tok::Ident(_)));
            let name = match p.tok()? {
                Tok::Ident(_) | Tok::Int(..) => p.bump(),
                _ => return None,
            };
            if p.is(Tok::PathSep) {
                return None; // Generic argument layout is not modeled here.
            }
            if named && p.open(Delim::Paren) {
                let args = arguments(p)?;
                value = expr(vec![value, word(format!(".{name}(")), comma_list(args), word(")")]);
            } else {
                // An unnamed field followed by `(args)` is Call(Field(...)),
                // not a method call: the next loop adds its own expression box.
                value = expr(vec![value, word(format!(".{name}"))]);
            }
        } else if p.open(Delim::Bracket) {
            p.i += 1;
            let index = parse_expr(p, Prec::Jump)?;
            p.expect_close(Delim::Bracket)?;
            value = expr(vec![value, word("["), index, word("]")]);
        } else if p.eat(Tok::Question) {
            value = expr(vec![value, word("?")]);
        } else {
            return Some(value);
        }
        value = bounded(value)?;
    }
}

fn arguments(p: &mut P) -> Option<Vec<Doc>> {
    p.i += 1;
    p.list(Delim::Paren, |p| parse_expr(p, Prec::Jump)).map(|(args, _)| args)
}

fn primary(p: &mut P) -> Option<Doc> {
    match p.peek()? {
        K::Tok(Tok::Int(..) | Tok::Float(_) | Tok::Str(..) | Tok::ByteStr(_) | Tok::ByteChar(_) | Tok::Char(..) | Tok::True | Tok::False) => {
            Some(expr(vec![word(p.bump())]))
        }
        K::Tok(Tok::Ident(_) | Tok::SelfLower | Tok::SelfUpper | Tok::Super | Tok::Crate | Tok::PathSep) => {
            let (path, generics) = p.path(true)?;
            if generics || p.is(Tok::Bang) {
                return None;
            }
            if p.open(Delim::Brace) {
                record(p, path)
            } else {
                Some(expr(vec![word(path)]))
            }
        }
        K::Open(Delim::Paren) => {
            p.i += 1;
            let (mut items, trailing) = p.list(Delim::Paren, |p| parse_expr(p, Prec::Jump))?;
            if items.len() == 1 && !trailing {
                Some(expr(vec![word("("), items.pop()?, word(")")]))
            } else {
                let close = if items.len() == 1 { ",)" } else { ")" };
                Some(expr(vec![word("("), comma_list(items), word(close)]))
            }
        }
        K::Open(Delim::Bracket) => {
            p.i += 1;
            let body = if p.close(Delim::Bracket) {
                p.i += 1;
                vec![word("[]")]
            } else {
                let first = parse_expr(p, Prec::Jump)?;
                if p.eat(Tok::Semi) {
                    let count = parse_expr(p, Prec::Jump)?;
                    p.expect_close(Delim::Bracket)?;
                    vec![word("["), first, word(";"), space(), count, word("]")]
                } else {
                    let mut items = vec![first];
                    if p.eat(Tok::Comma) {
                        items.extend(p.list(Delim::Bracket, |p| parse_expr(p, Prec::Jump))?.0);
                    } else {
                        p.expect_close(Delim::Bracket)?;
                    }
                    vec![word("["), comma_list(items), word("]")]
                }
            };
            Some(expr(vec![group(4, false, body)]))
        }
        // Blocks, attributes, macros and interpolated fragments retain the
        // existing explicit reject until their complete layout is modeled.
        _ => None,
    }
}

fn record(p: &mut P, path: String) -> Option<Doc> {
    p.i += 1;
    let mut fields = Vec::new();
    let mut rest = None;
    while !p.close(Delim::Brace) {
        if p.eat(Tok::DotDot) {
            rest = Some(parse_expr(p, Prec::Jump)?);
            break;
        }
        let name = match p.tok()? {
            Tok::Ident(_) | Tok::Int(..) => p.bump(),
            _ => return None,
        };
        let field = if p.eat(Tok::Colon) {
            vec![word(format!("{name}: ")), parse_expr(p, Prec::Jump)?]
        } else {
            vec![expr(vec![word(name)])]
        };
        fields.push(field);
        if !p.eat(Tok::Comma) && !p.close(Delim::Brace) {
            return None;
        }
    }
    p.expect_close(Delim::Brace)?;
    if fields.is_empty() && rest.is_none() {
        return Some(expr(vec![word(format!("{path} {{}}"))]));
    }
    let mut body = vec![space()];
    let count = fields.len();
    for (index, field) in fields.into_iter().enumerate() {
        body.extend(field);
        if index + 1 != count || rest.is_some() {
            body.push(word(","));
            body.push(space());
        } else {
            body.push(Doc::Break { spaces: 1, offset: -4, comma: true });
        }
    }
    if let Some(rest) = rest {
        body.push(word(".."));
        body.push(rest);
        body.push(Doc::Break { spaces: 1, offset: -4, comma: false });
    }
    Some(expr(vec![word(format!("{path} {{")), group(0, true, body), word("}")]))
}

enum Token {
    Word(String),
    Break { spaces: usize, offset: isize, comma: bool },
    Begin { indent: isize, consistent: bool },
    End,
}

fn tokens(doc: Doc, out: &mut Vec<Token>) {
    match doc {
        Doc::Word(text) => out.push(Token::Word(text)),
        Doc::Break { spaces, offset, comma } => out.push(Token::Break { spaces, offset, comma }),
        Doc::Group { indent, consistent, parts, .. } => {
            out.push(Token::Begin { indent, consistent });
            for part in parts {
                tokens(part, out);
            }
            out.push(Token::End);
        }
        Doc::Limit => unreachable!("bounded layout document"),
    }
}

fn render(doc: Doc) -> Option<String> {
    let doc = bounded(doc)?;
    let mut stream = Vec::new();
    tokens(doc, &mut stream);
    let mut sizes = vec![0usize; stream.len()];
    let mut starts = vec![0usize; stream.len()];
    let mut stack: Vec<usize> = Vec::new();
    let mut total = 0usize;
    // End markers are deferred until the next break (or EOF). A suffix
    // immediately after a closed box is unbreakable from that box, so it
    // participates in its width. This matches the pretty printer's SCAN
    // stack; measuring only through the matching End would wrap too late.
    let close = |stack: &mut Vec<usize>, sizes: &mut [usize], starts: &[usize], total: usize| {
        let mut depth = 0usize;
        while let Some(&at) = stack.last() {
            match stream[at] {
                Token::Begin { .. } => {
                    if depth == 0 {
                        break;
                    }
                    stack.pop();
                    sizes[at] = total - starts[at];
                    depth -= 1;
                }
                Token::End => {
                    stack.pop();
                    depth += 1;
                }
                Token::Break { .. } => {
                    stack.pop();
                    sizes[at] = total - starts[at];
                    if depth == 0 {
                        break;
                    }
                }
                Token::Word(_) => unreachable!("words are not deferred layout boundaries"),
            }
        }
    };
    for (index, token) in stream.iter().enumerate() {
        match token {
            Token::Word(text) => total += text.len(),
            Token::Begin { .. } => {
                starts[index] = total;
                stack.push(index);
            }
            Token::Break { spaces, .. } => {
                close(&mut stack, &mut sizes, &starts, total);
                starts[index] = total;
                stack.push(index);
                total += spaces;
            }
            Token::End => stack.push(index),
        }
    }
    close(&mut stack, &mut sizes, &starts, total);
    let mut out = String::new();
    let mut remaining = MARGIN as isize;
    let mut indent = 0isize;
    let mut pending = 0usize;
    let mut frames: Vec<Option<(isize, bool)>> = Vec::new();
    for (index, token) in stream.into_iter().enumerate() {
        match token {
            Token::Word(text) => {
                if out.len().saturating_add(pending).saturating_add(text.len()) > MAX_OUTPUT {
                    return None;
                }
                out.extend(std::iter::repeat_n(' ', pending));
                pending = 0;
                remaining -= text.len() as isize;
                out.push_str(&text);
            }
            Token::Begin { indent: offset, consistent } => {
                if sizes[index] as isize > remaining {
                    frames.push(Some((indent, consistent)));
                    indent += offset;
                } else {
                    frames.push(None);
                }
            }
            Token::End => {
                if let Some((previous, _)) = frames.pop().expect("balanced layout frame") {
                    indent = previous;
                }
            }
            Token::Break { spaces, offset, comma } => {
                let fits = match frames.last().copied().flatten() {
                    None => true,
                    Some((_, true)) => false,
                    Some((_, false)) => sizes[index] as isize <= remaining,
                };
                if fits {
                    pending += spaces;
                    remaining -= spaces as isize;
                } else {
                    if out.len() + usize::from(comma) + 1 > MAX_OUTPUT {
                        return None;
                    }
                    if comma {
                        out.push(',');
                    }
                    out.push('\n');
                    let next_indent = indent + offset;
                    if next_indent < 0 {
                        return None;
                    }
                    pending = next_indent as usize;
                    remaining = (MARGIN as isize - next_indent).max(60);
                }
            }
        }
    }
    Some(out)
}
