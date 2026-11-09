// expect: L0720
// rustc E0034: `Self::f` in an inherent impl, with two traits giving
// `S` an `f`.
trait Tr { fn f(&self) -> i32 { 1 } }
trait Tr2 { fn f(&self) -> i32 { 300 } }
struct S(i32);
impl Tr for S {}
impl Tr2 for S { fn f(&self) -> i32 { 2 } }
impl S { fn k(&self) -> i32 { let g = Self::f; g(self) } }
fn main() { println!("{}", S(3).k()); }
