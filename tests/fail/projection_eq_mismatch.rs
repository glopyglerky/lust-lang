// expect: L0110
// rustc E0271: B::Item must equal A::Item.
trait Array { type Item; }
impl<T> Array for [T; 4] { type Item = T; }
impl<T> Array for [T; 2] { type Item = T; }
fn same<A: Array, B: Array<Item = A::Item>>(a: A::Item, b: B::Item) -> bool where A::Item: PartialEq { a == b }
fn main() { println!("{}", same::<[u8; 4], [u16; 2]>(3, 3)); }
