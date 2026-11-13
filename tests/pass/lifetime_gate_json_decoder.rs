// Milestone 6 gate shape: a decoder over a borrowed byte buffer whose
// tokens borrow from the buffer.
#[derive(Debug)]
enum Token<'a> {
    Str(&'a [u8]),
    Num(u32),
    Punct(u8),
}

struct JsonDecoder<'a> {
    data: &'a [u8],
    at: usize,
}

impl<'a> JsonDecoder<'a> {
    fn new(data: &'a [u8]) -> JsonDecoder<'a> {
        JsonDecoder { data, at: 0 }
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        while self.at < self.data.len() && self.data[self.at] == b' ' {
            self.at += 1;
        }
        if self.at >= self.data.len() {
            return None;
        }
        let c = self.data[self.at];
        if c == b'"' {
            let start = self.at + 1;
            let mut end = start;
            while end < self.data.len() && self.data[end] != b'"' {
                end += 1;
            }
            self.at = end + 1;
            return Some(Token::Str(&self.data[start..end]));
        }
        if c >= b'0' && c <= b'9' {
            let mut n = 0u32;
            while self.at < self.data.len() && self.data[self.at] >= b'0' && self.data[self.at] <= b'9' {
                n = n * 10 + (self.data[self.at] - b'0') as u32;
                self.at += 1;
            }
            return Some(Token::Num(n));
        }
        self.at += 1;
        Some(Token::Punct(c))
    }
}

fn main() {
    let src = String::from("{\"k\": 12, \"v\": \"x\"}");
    let mut d = JsonDecoder::new(src.as_bytes());
    let mut toks = Vec::new();
    while let Some(t) = d.next_token() {
        toks.push(t);
    }
    println!("{}", toks.len());
    for t in &toks {
        match t {
            Token::Str(s) => println!("str {}", s.len()),
            Token::Num(n) => println!("num {}", n),
            Token::Punct(c) => println!("punct {}", c),
        }
    }
}
