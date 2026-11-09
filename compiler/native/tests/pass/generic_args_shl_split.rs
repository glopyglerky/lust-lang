// A '<<' that opens generic arguments with a qualified path splits in two.
trait Tr { type Item; }
struct W<T>(T);
impl Tr for u8 { type Item = u16; }
fn f<T: Tr>(x: W<<T as Tr>::Item>) -> W<<T as Tr>::Item> { x }
fn main() { let v: Vec<<u8 as Tr>::Item> = Vec::<<u8 as Tr>::Item>::new(); println!("{} {}", v.len(), f::<u8>(W(3)).0); }
