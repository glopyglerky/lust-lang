// expect: L0104
// A generic body sees only its bounds: the inherent method of the
// concrete type is not a candidate there (rustc E0599).
struct S(u8);
impl S { fn extra(&self) -> u8 { self.0 } }
trait G { fn get(&self) -> u8; }
impl G for S { fn get(&self) -> u8 { self.0 } }
fn g<T: G>(t: &T) -> u8 { t.get() + t.extra() }
fn main() { println!("{}", g(&S(1))); }
