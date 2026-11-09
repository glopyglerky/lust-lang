// expect: L0612
// rustc E0271: core's item bound: IntoIter must yield Item.
struct W { v: Vec<u8> }
impl IntoIterator for W { type Item = u16; type IntoIter = std::vec::IntoIter<u8>; fn into_iter(self) -> std::vec::IntoIter<u8> { self.v.into_iter() } }
fn main() { let w = W { v: vec![1] }; println!("{}", w.v.len()); }
