// expect: L0110
// rustc E0308: the Borrowed payload is a &str, not a String.
use std::borrow::Cow;

const C: Cow<'static, str> = Cow::Borrowed(1);

fn main() {
    println!("{}", C);
}
