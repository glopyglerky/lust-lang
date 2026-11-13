// Milestone 6 gate shape (ARCH 9.3 W-BORROW): a parser over borrowed input
// that hands out slices of it.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Parser<'a> {
        Parser { input, pos: 0 }
    }

    fn at_space(&self) -> bool {
        &self.input[self.pos..self.pos + 1] == " "
    }

    fn skip_spaces(&mut self) {
        while self.pos < self.input.len() && self.at_space() {
            self.pos += 1;
        }
    }

    /// The next word, borrowed from the input (not from the parser).
    fn word(&mut self) -> Option<&'a str> {
        self.skip_spaces();
        let start = self.pos;
        while self.pos < self.input.len() && !self.at_space() {
            self.pos += 1;
        }
        if start == self.pos { None } else { Some(&self.input[start..self.pos]) }
    }
}

fn words<'a>(input: &'a str) -> Vec<&'a str> {
    let mut p = Parser::new(input);
    let mut out = Vec::new();
    while let Some(w) = p.word() {
        out.push(w);
    }
    out
}

fn main() {
    let text = String::from("let x = 42 ;");
    let ws = words(&text);
    println!("{:?} {}", ws, ws.len());
}
