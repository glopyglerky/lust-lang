// expect: L0306
// rustc E0597: the item borrows d, which drops at the block end.
struct Bag(Vec<u32>);
impl<'a> IntoIterator for &'a Bag {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;
    fn into_iter(self) -> Self::IntoIter { println!("into_iter bag {}", self.0.len()); self.0.iter() }
}
fn main() { let r = { let d = Some(Bag(vec![1])); d.iter().flatten().next() }; println!("{:?}", r); }
