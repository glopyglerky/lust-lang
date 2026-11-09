// expect: L0001
// ARCH 2.8, rustc E0178: after `&`, a dyn type takes one bound; `&(dyn A + B)` needs parens.
trait Tr { fn m(&self) -> u8; }
fn f(_x: &dyn Tr + Send) {}
fn main() {}
