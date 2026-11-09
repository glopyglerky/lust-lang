// expect: L0610
// Two Conv impls fit; rustc E0283 (annotations needed). Pin from the opus-fixes review (q06).
trait Conv<T> { fn conv(&self) -> T; }
struct S;
impl Conv<u16> for S { fn conv(&self) -> u16 { 16 } }
impl Conv<u32> for S { fn conv(&self) -> u32 { 32 } }
fn main() { let a: u16 = S.conv() + S.conv(); println!("{}", a); }
