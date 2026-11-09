// expect: L0115
// A float part of a const pattern is an opaque value: it never covers the type.
const A: (f64, bool) = (1.0, true);
fn f(x: (f64, bool)) -> u8 {
    match x {
        A => 1,
        (_, false) => 2,
    }
}
fn main() {
    println!("{}", f((1.0, true)));
}
