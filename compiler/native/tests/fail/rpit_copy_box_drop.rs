// expect: L0312
// rustc E0506: a `Box` of an `impl Copy` value has drop glue, and the
// drop check assumes the opaque value needs every region it captures,
// so the borrow lasts until `b` drops.
struct C {
    n: u8,
}
fn mk(c: &C) -> impl Copy + '_ {
    &c.n
}
fn main() {
    let mut c = C { n: 0 };
    let b = Box::new(mk(&c));
    c.n = 1;
    println!("{}", c.n);
}
