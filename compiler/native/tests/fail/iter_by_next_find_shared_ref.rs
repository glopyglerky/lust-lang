// expect: L0308
// rustc E0596: `find` takes `&mut self`; `i` is a shared reference.
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
fn f<I: Iterator<Item = u32>>(i: &I) -> Option<u32> {
    i.find(|x| *x > 1)
}
fn main() {
    let n = N(0);
    println!("{:?}", f(&n));
}
