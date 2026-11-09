// expect: L0631
// rustc E0015: todo! with a message uses the formatting machinery.
const fn f() -> u8 { todo!("later") }
fn main() { println!("ok"); }
