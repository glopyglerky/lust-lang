// expect: L0104
// rustc E0599: the hidden type's inherent method is not visible through impl Trait (W-CHECK-G3).
trait Shape { fn area(&self) -> i32; }
struct Sq(i32);
impl Shape for Sq { fn area(&self) -> i32 { self.0 } }
impl Sq { fn side(&self) -> i32 { self.0 } }
fn make() -> impl Shape { Sq(3) }
fn main() { println!("{}", make().side()); }
