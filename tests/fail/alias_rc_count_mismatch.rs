// expect: L0110
// rustc E0308: the alias fixes `Rc<u8>`; the argument is `&Rc<u16>`.
use std::rc::Rc;
type R8 = Rc<u8>;
fn main() { let r = Rc::new(5u16); println!("{}", R8::strong_count(&r)); }
