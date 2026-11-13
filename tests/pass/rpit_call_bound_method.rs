// Callers of impl Trait see only its bounds: a trait method is called through the bound even where the hidden type has an inherent method of the same name (W-CHECK-G3).
trait Shape { fn area(&self) -> i32; }
struct Sq { side: i32 }
impl Shape for Sq { fn area(&self) -> i32 { self.side * self.side } }
impl Sq { fn area(&self) -> i32 { -1 } }
fn make() -> impl Shape { Sq { side: 3 } }
fn main() { let m = make(); println!("{}", m.area()); println!("{}", make().area()); }
