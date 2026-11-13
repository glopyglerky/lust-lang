// expect: L0306
// rustc E0597: the callee passes a `Parser` over a local string where the closure's annotation
// and the enclosing method's bound need the impl's `'a`.
struct Parser<'a> { src: &'a str, pos: usize }
impl<'a> Parser<'a> { fn idx(&mut self, mut f: impl FnMut(&mut Parser<'a>, i32) -> u32) -> u32 { let s = String::from("zz"); let mut o = Parser { src: &s, pos: 0 }; f(&mut o, 0) } fn list(&mut self, mut pe: impl FnMut(&mut Parser<'a>) -> u32) -> u32 { self.idx(|p: &mut Parser<'a>, _: i32| pe(p)) } }
fn main() { let s = String::from("ab"); let mut p = Parser { src: &s, pos: 0 }; println!("{}", p.list(|q| q.src.len() as u32)); }
