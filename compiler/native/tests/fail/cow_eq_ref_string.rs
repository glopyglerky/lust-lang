// expect: L0113
// rustc E0277: no PartialEq<&String> for Cow<str>.
use std::borrow::Cow;

fn main() {
    let c: Cow<str> = Cow::Borrowed("x");
    let st = String::from("x");
    println!("{}", c == &st);
}
