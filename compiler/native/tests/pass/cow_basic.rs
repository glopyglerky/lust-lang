// std::borrow::Cow: constructors with and without an expected type,
// patterns by reference and by value, Display, Debug (the inner value,
// no variant name), Clone, ==, and Cow<[u8]>.
use std::borrow::Cow;

fn clean(s: &str) -> Cow<'_, str> {
    if s.contains(" ") {
        Cow::Owned(s.replace(" ", "_"))
    } else {
        Cow::Borrowed(s)
    }
}

fn describe(c: &Cow<'_, str>) -> &'static str {
    match c {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}

fn main() {
    let a = clean("hello");
    let b = clean("a b c");
    println!("{} {}", describe(&a), describe(&b));
    println!("{} {} {:?} {:?}", a, b, a, b);
    match b {
        Cow::Owned(s) => println!("owned {} {}", s, s.len()),
        Cow::Borrowed(s) => println!("borrowed {}", s),
    }
    let c: Cow<'_, str> = Cow::Borrowed("x");
    let d = c.clone();
    println!("{} {}", c == d, a == clean("hello"));
    let e: Cow<'static, [u8]> = Cow::Owned(vec![1, 2, 3]);
    println!("{:?}", e);
}
