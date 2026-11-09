// expect: L0733
// rustc: lifetime may not live long enough (`Box<dyn T>` is `'static`).
trait T { fn v(&self) -> usize; }
struct R<'a>(&'a str);
impl<'a> T for R<'a> { fn v(&self) -> usize { self.0.len() } }
fn mk(s: &str) -> Box<dyn T> { Box::new(R(s)) }
fn main() { println!("{}", mk("abc").v()); }
