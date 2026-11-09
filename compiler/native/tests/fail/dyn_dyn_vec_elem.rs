// expect: L0110
// ARCH 2.8, rustc E0277: `Vec<T>` needs a sized `T`; `dyn Tr` is not.
trait Tr { fn m(&self) -> u8; }
fn main() { let v: Vec<dyn Tr> = Vec::new(); }
