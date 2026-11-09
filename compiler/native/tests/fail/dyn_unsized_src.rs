// expect: L0110
// ARCH 2.8, rustc E0277: only a sized type unsizes to `&dyn`; `str` is not sized.
trait Tr { fn m(&self) -> u8; }
impl Tr for str { fn m(&self) -> u8 { 1 } }
fn main() { let s: &str = "hi"; let d: &dyn Tr = s; println!("{}", d.m()); }
