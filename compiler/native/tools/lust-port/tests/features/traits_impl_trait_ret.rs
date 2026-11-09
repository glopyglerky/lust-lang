// feature: traits.impl_trait_ret
// form: impl Trait in return position
trait Shape { fn area(&self) -> i32; }
struct Sq { side: i32 }
impl Shape for Sq { fn area(&self) -> i32 { self.side * self.side } }
fn make() -> impl Shape { Sq { side: 3 } }
fn main() { println!("{}", make().area()); }
