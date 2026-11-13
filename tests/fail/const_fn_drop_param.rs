// expect: L0631
// rustc E0493: the parameter is destroyed inside the const fn.
struct D(i32);
impl Drop for D { fn drop(&mut self) {} }
const fn h(d: D) -> i32 { d.0 }
fn main() { println!("{}", h(D(3))); }
