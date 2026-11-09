// expect: L0720
// rustc E0034: `S::f` as a value names two traits' `f` (Tr's written
// impl fn and Tr2's default body); hunt6 01:11.
trait Tr { fn f(&self) -> i32; }
trait Tr2 { fn f(&self) -> i32 { 300 } }
struct S(i32);
impl Tr for S { fn f(&self) -> i32 { self.0 * 2 } }
impl Tr2 for S {}
fn main() {
    let g = S::f;
    println!("{}", g(&S(3)));
}
