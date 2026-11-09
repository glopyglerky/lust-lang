// expect: L0107
// rustc E0107: an `impl Trait` argument takes no written generic argument,
// as a call or as a fn value.
use std::fmt::Display;
fn show(x: impl Display) -> String {
    format!("<{}>", x)
}
fn main() {
    let f = show::<u8>;
    println!("{}", f(1));
}
