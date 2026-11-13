// An `impl Trait` local that is moved out before it is written again
// drops nothing at the write, so its old borrows end (rustc accepts).
// The other write forms that rustc accepts are here too.
struct C {
    v: Vec<u32>,
    n: u8,
}
fn items(c: &C) -> impl Iterator<Item = &u32> {
    c.v.iter()
}
fn main() {
    let mut c = C { v: vec![1, 2], n: 0 };
    let c0 = C { v: vec![7], n: 0 };
    let r = &c;
    let mut it = Some(items(r));
    let n = it.take().map_or(0, |i| i.count());
    drop(it);
    it = None;
    c.n = n as u8;
    println!("{} {}", c.n, it.is_none());
    let r = &c;
    let mut it2 = items(r);
    it2 = items(&c0);
    let s: u32 = it2.sum();
    c.n = s as u8;
    println!("{}", c.n);
}
