// A plain binding of the same type as an `impl Trait` hidden type is not
// an opaque value: its drop uses no borrow (rustc decides from the
// binding's own type, not from the hidden type of another value).
struct C {
    v: Vec<u32>,
    n: u8,
}
fn mk(v: &Vec<u32>) -> impl Iterator<Item = &u32> {
    v.iter()
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    let other = vec![4u32];
    let total: u32 = mk(&other).sum();
    let p = c.v.iter();
    let _ = &p;
    c.v.push(total);
    println!("{} {}", c.v.len(), c.n);
}
