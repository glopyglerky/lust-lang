// expect: L0113
// rustc E0277: Cow<str> orders only against Cow<str>.
use std::borrow::Cow;

fn main() {
    let c: Cow<str> = Cow::Borrowed("x");
    println!("{}", c < "y");
}
