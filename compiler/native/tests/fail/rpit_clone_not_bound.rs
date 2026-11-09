// expect: L0104
// rustc E0599: Clone is not in the bounds, so the hidden type's Clone is not visible (W-CHECK-G3).
trait Shape { fn area(&self) -> i32; }
#[derive(Clone)]
struct Sq(i32);
impl Shape for Sq { fn area(&self) -> i32 { self.0 } }
fn make() -> impl Shape { Sq(3) }
fn main() { let s = make(); let t = s.clone(); println!("{}", t.area()); }
