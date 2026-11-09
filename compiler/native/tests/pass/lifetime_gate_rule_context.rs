// Milestone 6 gate shape (ts-rust `RuleContext<'a, 'c>`): a context holding
// `&'a mut TypeParser<'c>`, so `'c` is invariant through the `&mut`.
struct TypeParser<'c> {
    src: &'c str,
    seen: Vec<&'c str>,
}

struct RuleContext<'a, 'c> {
    parser: &'a mut TypeParser<'c>,
    depth: u32,
}

impl<'a, 'c> RuleContext<'a, 'c> {
    fn new(parser: &'a mut TypeParser<'c>) -> RuleContext<'a, 'c> {
        RuleContext { parser, depth: 0 }
    }

    fn note(&mut self, at: usize, len: usize) {
        let piece = &self.parser.src[at..at + len];
        self.parser.seen.push(piece);
        self.depth += 1;
    }
}

fn main() {
    let text = String::from("abcdef");
    let mut tp = TypeParser { src: &text, seen: Vec::new() };
    {
        let mut cx = RuleContext::new(&mut tp);
        cx.note(0, 2);
        cx.note(2, 3);
        println!("{}", cx.depth);
    }
    println!("{:?}", tp.seen);
}
