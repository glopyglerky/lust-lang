// expect: L0303
// rustc E0502: += borrows the Cow mutably while a view of it lives.
use std::borrow::Cow;

fn main() {
    let mut e: Cow<str> = Cow::Borrowed("m");
    let r: &str = &e;
    e += "n";
    println!("{} {}", r, e);
}
