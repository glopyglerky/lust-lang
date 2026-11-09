// expect: L0110
// rustc E0308: two fns' impl Trait types are different types (W-CHECK-G3).
trait Shape { fn area(&self) -> i32; }
struct Sq(i32);
impl Shape for Sq { fn area(&self) -> i32 { self.0 } }
fn make() -> impl Shape { Sq(3) }
fn make2() -> impl Shape { Sq(4) }
fn main() { let mut s = make(); s = make2(); println!("{}", s.area()); }
