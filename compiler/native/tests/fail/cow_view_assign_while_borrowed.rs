// expect: L0312
// rustc E0506: the Cow is assigned while a view method's result borrows it.
use std::borrow::Cow;

fn main() {
    let mut c: Cow<str> = Cow::Borrowed("x");
    let t = c.trim();
    c = Cow::Owned(String::new());
    println!("{t} {c}");
}
