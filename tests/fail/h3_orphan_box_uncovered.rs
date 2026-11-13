// expect: L0616
// rustc E0210: Box is fundamental and does not cover T.
struct Local;
impl<T> PartialEq<Local> for Box<T> { fn eq(&self, _: &Local) -> bool { true } }
fn main() { println!("{}", Box::new(1u8) == Local); }
