// expect: L0733
// rustc: lifetime may not live long enough. The bound's two elided
// lifetimes are distinct, so the closure cannot store one in the other.
fn run(f: impl Fn(&mut &u8, &u8)) {
    let a = 0u8;
    let mut r = &a;
    let x = 1u8;
    f(&mut r, &x);
    println!("{}", r);
}
fn main() {
    run(|r, x| *r = x);
}
