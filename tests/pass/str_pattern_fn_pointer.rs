// A `fn(char) -> bool` pointer as a str pattern (ts-rust jsdoc.rs
// `trim_part_end`): every Pattern method takes it, as a closure.
use std::borrow::Cow;
fn is_ws(c: char) -> bool { c == ' ' || c == '*' }
fn trim_part_end(part: &mut Cow<'_, str>, f: fn(char) -> bool) {
    match part {
        Cow::Borrowed(s) => *s = (*s).trim_end_matches(f),
        Cow::Owned(s) => s.truncate(s.trim_end_matches(f).len()),
    }
}
fn main() {
    let mut a: Cow<str> = Cow::Borrowed("ab * ");
    trim_part_end(&mut a, is_ws);
    let mut b: Cow<str> = Cow::Owned(String::from("cd**"));
    trim_part_end(&mut b, char::is_alphabetic);
    println!("{:?} {:?}", a, b);
    let f: fn(char) -> bool = is_ws;
    println!("{:?}", "x y*z".split(f).collect::<Vec<_>>());
    println!("{:?} {:?} {:?}", "  hi  ".trim_start_matches(f), "a*b".find(f), "a*b".rsplit_once(f));
    println!("{:?} {}", "q*r s".matches(f).count(), "q r".replace(f, "_"));
    println!("{:?}", "ab cd".rsplit(f).collect::<Vec<_>>());
    let g: fn(char) -> bool = |c| c.is_ascii_digit();
    println!("{:?} {}", "a1b2".trim_matches(g), "12ab".starts_with(g));
}
