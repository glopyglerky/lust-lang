// feature: closures.fn_ptr_lifetime
// form: fn pointer types that name a lifetime of the enclosing impl, as in ts-rust frontend/parser/parser_p2.rs
struct Parser<'a> { src: &'a str, pos: usize }
type Node = usize;
impl<'a> Parser<'a> {
    fn a(&mut self) -> Node { self.pos += 1; self.pos }
    fn b(&mut self) -> Node { self.pos += 2; self.src.len() + self.pos }
    fn list(&mut self, init: bool) -> Node {
        let parse_element: fn(&mut Parser<'a>) -> Node = if init { Parser::a } else { Parser::b };
        self.delimited(parse_element)
    }
    fn delimited(&mut self, mut parse_element: impl FnMut(&mut Parser<'a>) -> Node) -> Node { parse_element(self) + parse_element(self) }
    fn union(&mut self, parse_constituent_type: fn(&mut Parser<'a>) -> Node) -> Node { parse_constituent_type(self) }
}
fn main() {
    let s = String::from("xyz");
    let mut p = Parser { src: &s, pos: 0 };
    println!("{} {} {}", p.list(true), p.list(false), p.union(Parser::b));
}
