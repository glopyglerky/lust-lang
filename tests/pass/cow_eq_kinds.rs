// Cow<str> against str, &str, String, and Cow<str>, both ways and
// behind equal references, as std's PartialEq impls allow (W-STD-B).
use std::borrow::Cow;

fn pick(s: &str, owned: bool) -> Cow<'_, str> {
    if owned { Cow::Owned(s.to_string()) } else { Cow::Borrowed(s) }
}

fn main() {
    let a = pick("ab", true);
    let b = pick("ab", false);
    let st = String::from("ab");
    let s: &str = "ab";
    println!("{} {} {} {}", a == b, a == s, s == a, a == *s);
    println!("{} {} {}", *s == a, a == st, st == a);
    println!("{} {}", &a == s, &&a == &s);
    println!("{} {} {}", a != "x", "x" != b, b != st);
    println!("{} {} {}", a < b, a <= b, pick("b", false) > a);
    let v = vec![pick("x", false), pick("y", true)];
    println!("{}", v.iter().filter(|c| **c == "y").count());
    println!("{:?}", v.iter().position(|c| *c == "x"));
}
