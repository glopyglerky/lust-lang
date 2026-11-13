// expect: L0112
trait Plain { fn n(&self) -> u8; }
#[derive(Debug)]
struct M { v: Box<dyn Plain> }
fn main() {}
