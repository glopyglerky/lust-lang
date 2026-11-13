// expect: L0303
// rustc E0502: the item borrows bags; push needs it mutably.
struct Bag(Vec<u32>);
impl<'a> IntoIterator for &'a Bag {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;
    fn into_iter(self) -> Self::IntoIter { println!("into_iter bag {}", self.0.len()); self.0.iter() }
}
fn main() { let mut bags = vec![Bag(vec![1])]; let x = bags.iter().flatten().next().unwrap(); bags.push(Bag(vec![])); println!("{}", x); }
