// Comparison method calls across std types (rustc selects
// `Self: PartialEq<R>` with `R` the argument's pointee): the string types
// and the sequence forms, by value, through references, in closures and
// loops.
use std::borrow::Cow;

fn main() {
    let s = String::from("a");
    println!("{} {}", s.eq("a"), s.ne("b"));
    let r: &str = "a";
    println!("{} {}", s.eq(&r), r.eq(&s));
    let c: Cow<str> = Cow::Borrowed("a");
    println!("{} {} {}", c.eq("a"), c.eq(&s), "a".eq(&c));
    let v = vec![1, 2];
    println!("{} {} {}", v.eq(&[1, 2]), v.eq(&v[..]), v[..].eq(&v));
    let a = [1u8, 2];
    let w: Vec<u8> = vec![1, 2];
    println!("{} {} {}", a.eq(&w[..]), a.ne(&[3u8, 4][..]), w.ne(&a));
    let f = |t: &str| s.eq(t);
    println!("{} {}", f("a"), f("b"));
    let mut n = 0;
    for x in ["a", "b", "a"] {
        if s.eq(x) {
            n += 1;
        }
    }
    println!("{n}");
    let rr = &&s;
    println!("{}", rr.eq(&"a"));
}
