// Flatten::last is core's FlattenCompat::last: each inner last, then the older answer drops; inner iterators drop after their own last.
#[derive(Debug)]
struct D(u32);
impl Drop for D { fn drop(&mut self) { println!("drop D{}", self.0); } }
struct Noisy(Vec<D>, u32);
impl Iterator for Noisy {
    type Item = D;
    fn next(&mut self) -> Option<D> { if self.0.is_empty() { None } else { Some(self.0.remove(0)) } }
}
impl Drop for Noisy { fn drop(&mut self) { println!("drop iter {}", self.1); } }
fn main() {
    let c = [vec![], vec![D(3)], vec![D(0), D(9)]];
    println!("{:?}", c.into_iter().flatten().last().is_some());
    let mut it = vec![vec![D(1), D(2)], vec![D(4)], vec![]].into_iter().flatten();
    println!("{:?}", it.next());
    println!("{:?}", it.last());
    let n = vec![Noisy(vec![D(10), D(11)], 1), Noisy(vec![], 2), Noisy(vec![D(12)], 3)];
    println!("{:?}", n.into_iter().flatten().last());
    let e: Vec<Vec<D>> = vec![vec![], vec![]];
    println!("{:?}", e.into_iter().flatten().last());
    let s = vec![Some(D(5)), None, Some(D(6))];
    println!("{:?}", s.into_iter().flatten().last());
    let r: Vec<Vec<u8>> = vec![vec![1, 2], vec![3]];
    println!("{:?}", r.iter().flatten().last());
    println!("end");
}
