// expect: L0110
// rustc E0308: an impl Trait value is not its hidden type (W-CHECK-G3).
trait Shape { fn area(&self) -> i32; }
struct Sq(i32);
impl Shape for Sq { fn area(&self) -> i32 { self.0 } }
fn make() -> impl Shape { Sq(3) }
fn main() { let s: Sq = make(); println!("{}", s.area()); }
