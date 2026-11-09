// rustc-error: E0015
// lust-expect: L0631
// ARCH 2.13: a static initializer that calls a non-const fn is
// an error. Running it at startup is not a substitute.
// From the sol probe invalid_static.rs.
static S: String = String::from("x");
fn main() {
    println!("{}", S);
}
