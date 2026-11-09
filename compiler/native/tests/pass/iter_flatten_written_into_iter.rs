// flatten over items with a written IntoIterator impl (R15): core calls into_iter on each item, in order; generic impls and user iterators too.
struct Bag(Vec<u32>);
impl<'a> IntoIterator for &'a Bag {
    type Item = &'a u32;
    type IntoIter = std::slice::Iter<'a, u32>;
    fn into_iter(self) -> Self::IntoIter { println!("into_iter bag {}", self.0.len()); self.0.iter() }
}
struct G<T>(Vec<T>);
struct GIter<'a, T> { v: &'a [T], i: usize }
impl<'a, T> Iterator for GIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> { let x = self.v.get(self.i); self.i += 1; x }
}
impl<'a, T> IntoIterator for &'a G<T> {
    type Item = &'a T;
    type IntoIter = GIter<'a, T>;
    fn into_iter(self) -> GIter<'a, T> { GIter { v: &self.0, i: 0 } }
}
struct Own(Vec<String>);
impl IntoIterator for Own {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter { self.0.into_iter() }
}
fn main() {
    let bags = vec![Bag(vec![1, 2]), Bag(vec![]), Bag(vec![3])];
    let all: Vec<&u32> = bags.iter().flatten().collect();
    println!("{:?}", all);
    let one: Option<Bag> = Some(Bag(vec![7, 8]));
    for x in one.iter().flatten() { println!("x {x}"); }
    let mut it = bags.iter().flatten();
    println!("{:?} {:?}", it.next(), it.size_hint());
    let gs = [G(vec!['a', 'b']), G(vec!['c'])];
    let s: String = gs.iter().flatten().collect();
    println!("{s}");
    let owns = vec![Own(vec!["p".into(), "q".into()]), Own(vec!["r".into()])];
    let joined: Vec<String> = owns.into_iter().flatten().collect();
    println!("{:?}", joined);
    let n = bags.iter().flatten().filter(|x| **x > 1).count();
    println!("{n}");
}
