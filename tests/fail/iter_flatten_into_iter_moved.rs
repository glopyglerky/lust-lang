// expect: L0311
// rustc E0505: bags moves while the collected items borrow it.
struct Bag(Vec<u32>);
impl<'a> IntoIterator for &'a Bag {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;
    fn into_iter(self) -> Self::IntoIter { println!("into_iter bag {}", self.0.len()); self.0.iter() }
}
fn main() { let bags = vec![Bag(vec![1])]; let x: Vec<&u32> = bags.iter().flatten().collect(); drop(bags); println!("{:?}", x); }
