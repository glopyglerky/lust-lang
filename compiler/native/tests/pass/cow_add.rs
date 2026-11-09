// Cow<str> addition: `+` and `+=` with &str and Cow right sides,
// in operator and method form. A Borrowed becomes Owned with the
// to_mut capacity, then the push grows it as String does.
use std::borrow::Cow;
use std::ops::{Add, AddAssign};

fn main() {
    let a: Cow<str> = Cow::Borrowed("a");
    let b = a + "b";
    println!("{} {}", b, matches!(b, Cow::Owned(_)));
    let mut c: Cow<str> = Cow::Borrowed("x");
    c += "y";
    println!("{} {}", c, matches!(c, Cow::Owned(_)));
    let d: Cow<str> = Cow::Owned("p".to_string());
    println!("{}", d + "q");
    let e: Cow<str> = Cow::Borrowed("a");
    println!("{}", e + Cow::Borrowed("z"));
    let mut f: Cow<str> = Cow::Owned("m".to_string());
    f += Cow::Borrowed("n");
    println!("{} {}", f, f.clone().into_owned().capacity());
    let g: Cow<str> = Cow::Borrowed("a");
    println!("{}", g.add("b"));
    let mut h: Cow<str> = Cow::Borrowed("x");
    h.add_assign("y");
    println!("{}", h);
    let i: Cow<str> = Cow::Borrowed("p");
    println!("{}", i.add(Cow::Owned("q".to_string())));
    let big: Cow<str> = Cow::Borrowed("hello");
    let big2 = big + " world, this is a longer suffix";
    let cap = match &big2 {
        Cow::Owned(s) => s.capacity(),
        Cow::Borrowed(_) => 0,
    };
    println!("{} {}", big2, cap);
}
