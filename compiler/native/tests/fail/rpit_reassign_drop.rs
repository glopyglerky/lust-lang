// expect: L0312
// rustc E0506: overwriting a local that still holds an `impl Trait`
// value drops the old value in place, and rustc keeps the borrows it
// captured in scope while the local stays live.
struct C {
    v: Vec<u32>,
    n: u8,
}
fn items(c: &C) -> impl Iterator<Item = &u32> {
    c.v.iter()
}
fn main() {
    let mut c = C { v: vec![1, 2], n: 0 };
    let r = &c;
    let mut it = Some(items(r));
    it = None;
    c.n = 1;
    println!("{} {}", c.n, it.is_none());
}
