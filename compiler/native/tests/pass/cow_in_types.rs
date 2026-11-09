// Cow<'a, str> in a derived enum and a struct field with a HashSet of
// Cow keys, as the json kernel holds names.
use std::borrow::Cow;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
enum Token<'a> {
    Str(Cow<'a, str>),
    Num(i64),
}

struct Frame<'a> {
    names: Option<HashSet<Cow<'a, str>>>,
    last: Token<'a>,
}

fn scan<'a>(words: &[&'a str]) -> Vec<Token<'a>> {
    let mut out = Vec::new();
    for &w in words {
        if w == "12" {
            out.push(Token::Num(12));
        } else if w.contains("_") {
            out.push(Token::Str(Cow::Owned(w.replace("_", "-"))));
        } else {
            out.push(Token::Str(Cow::Borrowed(w)));
        }
    }
    out
}

fn main() {
    let src = vec!["a", "b_c", "12", "d", "a"];
    let toks = scan(&src);
    println!("{:?}", toks);
    let mut f = Frame { names: Some(HashSet::new()), last: Token::Num(0) };
    for t in &toks {
        if let Token::Str(s) = t {
            let dup = match &mut f.names {
                Some(names) => !names.insert(s.clone()),
                None => false,
            };
            println!("{} {}", s, dup);
        }
        f.last = t.clone();
    }
    println!("{:?} {}", f.last, toks[0] == Token::Str(Cow::Borrowed("a")));
    if let Some(n) = &f.names {
        println!("{}", n.len());
    }
}
