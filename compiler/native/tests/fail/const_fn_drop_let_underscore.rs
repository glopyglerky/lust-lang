// expect: L0631
// rustc E0493: `let _ =` destroys the value inside the const fn.
struct D(u8);
impl Drop for D { fn drop(&mut self) {} }
const fn f() -> usize { let _ = D(1); 4 }
fn main() { println!("{}", f()); }
