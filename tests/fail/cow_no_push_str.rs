// expect: L0104
// rustc E0599: no push_str on Cow<str> or str.
use std::borrow::Cow;

fn main() {
    let mut c: Cow<str> = Cow::Borrowed("x");
    c.push_str("y");
}
