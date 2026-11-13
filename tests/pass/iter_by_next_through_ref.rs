// find, nth, position, size_hint through `&I` and `&mut I` on user iterators
// and Iterator-bound parameters (W-STD-B).
struct N(u32);
impl Iterator for N {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 < 5 { self.0 += 1; Some(self.0) } else { None }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = (5 - self.0) as usize;
        (n, Some(n))
    }
}
fn hint_of<I: Iterator>(i: &I) -> (usize, Option<usize>) {
    i.size_hint()
}
fn find_in<I: Iterator<Item = u32>>(i: &mut I) -> Option<u32> {
    i.find(|x| x % 2 == 0)
}
fn main() {
    let mut n = N(0);
    println!("{:?} {:?}", hint_of(&n), find_in(&mut n));
    println!("{:?} {:?}", hint_of(&n), n.next());
    let mut v = vec![1u32, 2, 4].into_iter();
    println!("{:?} {:?} {:?}", hint_of(&v), find_in(&mut v), hint_of(&v));
    let r = &mut n;
    println!("{:?} {:?}", r.size_hint(), r.position(|x| x == 5));
}
