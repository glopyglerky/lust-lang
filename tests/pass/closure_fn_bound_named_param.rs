// ts-rust frontend/parser/parser_p1.rs parse_list: a closure annotated `&mut Parser<'a>` calls
// through the enclosing method's `impl FnMut(&mut Parser<'a>) -> u32` bound.
struct Parser<'a> { src: &'a str, pos: usize }
impl<'a> Parser<'a> {
    fn parse_list_index(&mut self, mut f: impl FnMut(&mut Parser<'a>, i32) -> u32) -> Vec<u32> {
        let mut out = Vec::new();
        while self.pos < self.src.len() { let v = f(self, out.len() as i32); out.push(v); self.pos += 1; }
        out
    }
    fn parse_list(&mut self, mut parse_element: impl FnMut(&mut Parser<'a>) -> u32) -> Vec<u32> {
        self.parse_list_index(|p: &mut Parser<'a>, _: i32| parse_element(p))
    }
}
fn main() {
    let s = String::from("abc");
    let mut p = Parser { src: &s, pos: 0 };
    let r = p.parse_list(|p| p.src.as_bytes()[p.pos] as u32);
    println!("{:?}", r);
}
