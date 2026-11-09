// W-BORROW3: fn pointer types that name a lifetime of the enclosing item,
// as in ts-rust frontend/parser/parser_p2.rs: `fn(&mut Parser<'a>) -> Node`
// in `let` annotations (from methods with the impl's early-bound lifetime,
// through `if`, reassigned), as a param passed on and called, at an
// `impl FnMut` bound, in a Vec; and in free fn signatures.
struct Parser<'a> { src: &'a str, pos: usize }
type Node = usize;
impl<'a> Parser<'a> {
    fn a(&mut self) -> Node { self.pos += 1; self.pos }
    fn b(&mut self) -> Node { self.pos += 2; self.src.len() + self.pos }
    fn list(&mut self, init: bool) -> Node {
        let parse_element: fn(&mut Parser<'a>) -> Node = if init { Parser::a } else { Parser::b };
        let first = parse_element(self);
        self.delimited(parse_element) + first
    }
    fn heritage(&mut self, iface: bool) -> Node {
        let mut parse_element: fn(&mut Parser<'a>) -> Node = Parser::a;
        if iface {
            parse_element = Parser::b;
        }
        self.delimited(parse_element)
    }
    fn delimited(&mut self, mut parse_element: impl FnMut(&mut Parser<'a>) -> Node) -> Node {
        parse_element(self) + parse_element(self)
    }
    fn union(&mut self, parse_constituent_type: fn(&mut Parser<'a>) -> Node) -> Node {
        let t = parse_constituent_type(self);
        t + self.inner(parse_constituent_type)
    }
    fn inner(&mut self, f: fn(&mut Parser<'a>) -> Node) -> Node { f(self) }
    fn all(&mut self) -> Vec<Node> {
        let fs: Vec<fn(&mut Parser<'a>) -> Node> = vec![Parser::a, Parser::b];
        fs.iter().map(|f| f(self)).collect()
    }
}
fn pick<'a>(f: fn(&'a str) -> &'a str, s: &'a str) -> &'a str { f(s) }
fn first(s: &str) -> &str { &s[..1] }
fn keep<'a>(f: fn(&'a str) -> usize, s: &'a str, out: &mut Vec<usize>) { out.push(f(s)) }
fn count(s: &str) -> usize { s.len() }
fn main() {
    let s = String::from("xyz");
    let mut p = Parser { src: &s, pos: 0 };
    println!("{} {}", p.list(true), p.list(false));
    println!("{} {}", p.heritage(true), p.heritage(false));
    println!("{}", p.union(Parser::a));
    println!("{:?}", p.all());
    let t = String::from("hello");
    let r = pick(first, &t);
    let mut out = Vec::new();
    keep(count, &t, &mut out);
    println!("{} {:?}", r, out);
}
