// expect: L0610
// rustc E0283: two impls can give the item type; nothing picks one.
trait Array { type Item; fn mk() -> Self::Item; }
impl Array for [u16; 4] { type Item = u16; fn mk() -> u16 { 3 } }
impl Array for [u8; 4] { type Item = u8; fn mk() -> u8 { 4 } }
fn make<A: Array>() -> A::Item { A::mk() }
fn main() { let x: u16 = make(); println!("{}", x); }
