// expect: L0312
// rustc E0506: `swap` moves the opaque value into `b`, which keeps its
// captured borrow of `c` until `b` drops at the end of `main`.
struct C {
    v: Vec<u32>,
    n: u8,
}
fn mk(c: &C) -> impl Iterator<Item = &u32> {
    c.v.iter()
}
fn main() {
    let mut c = C { v: vec![1], n: 0 };
    let mut a = Some(mk(&c));
    let mut b = None;
    std::mem::swap(&mut a, &mut b);
    drop(a);
    c.n = 1;
    println!("{}", c.v.len());
}
