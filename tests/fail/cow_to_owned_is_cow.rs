// expect: L0110
// rustc E0308: to_owned on a Cow is the Cow's own (a Cow, not a String).
use std::borrow::Cow;

fn main() {
    let c: Cow<str> = Cow::Borrowed("x");
    let s: String = c.to_owned();
    println!("{s}");
}
