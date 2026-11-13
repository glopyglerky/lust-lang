// expect: L0312
// rustc E0506: the opaque value captures `c` (edition 2024) even though
// its hidden type `u8` holds no borrow, and pushing it into `v` keeps
// that borrow while `v` lives to its drop (a `Vec` has drop glue).
struct C {
    n: u8,
}
fn mk(c: &C) -> impl Copy {
    c.n
}
fn main() {
    let mut c = C { n: 0 };
    let mut v = Vec::new();
    v.push(mk(&c));
    c.n = 1;
    println!("{}", c.n);
}
