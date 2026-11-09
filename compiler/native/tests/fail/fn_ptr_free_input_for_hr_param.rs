// expect: L0110
// rustc E0308: a pointer over the caller's `'x` is not one for every
// lifetime.
fn take<T>(f: fn(&u8) -> T, x: &u8) -> T {
    f(x)
}
fn id(a: &u8) -> &u8 {
    a
}
fn run<'x>(p: fn(&'x u8) -> &'x u8, x: &'x u8) -> u8 {
    *take(p, x)
}
fn main() {
    let x = 4u8;
    println!("{}", run(id, &x));
}
