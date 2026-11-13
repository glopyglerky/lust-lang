// expect: L0104
// rustc E0277: no From<&mut String> for Cow<str>.
use std::borrow::Cow;

fn main() {
    let mut st = String::new();
    let c: Cow<str> = (&mut st).into();
    println!("{c}");
}
