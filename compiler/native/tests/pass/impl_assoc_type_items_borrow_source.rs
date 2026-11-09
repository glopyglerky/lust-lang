// The trait's `Self::Tok` is the impl's `&'a str`: two items from one
// lexer borrow the text, not the lexer.
trait Lex { type Tok; fn next_tok(&mut self) -> Option<Self::Tok>; }
struct L<'a> { s: &'a str, i: usize }
impl<'a> Lex for L<'a> {
    type Tok = &'a str;
    fn next_tok(&mut self) -> Option<&'a str> {
        if self.i >= self.s.len() { return None; }
        let t = &self.s[self.i..self.i + 1];
        self.i += 1;
        Some(t)
    }
}
fn main() {
    let text = String::from("abc");
    let mut l = L { s: &text, i: 0 };
    let a = l.next_tok();
    let b = l.next_tok();
    println!("{:?} {:?}", a, b);
}
