// feature: lifetimes.named
// form: lifetime param on a struct with a ref field, as in ts-rust Parser<'a> and JsonDecoder<'a>
struct Parser<'a> { text: &'a str, pos: usize }
impl<'a> Parser<'a> {
    fn rest(&self) -> &'a str { &self.text[self.pos..] }
}
fn main() { let p = Parser { text: "hello", pos: 2 }; println!("{}", p.rest()); }
