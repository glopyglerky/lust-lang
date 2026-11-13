// Deref coercion from &Cow<str> and &Cow<[u8]> to &str and &[u8]: fn
// arguments, a let annotation, a Vec of &Cow, and a closure capture.
use std::borrow::Cow;

fn marker(s: &str) -> bool {
    s.contains("@")
}

fn first(b: &[u8]) -> u8 {
    b[0]
}

fn main() {
    let s: Cow<'_, str> = Cow::Borrowed("a@b");
    let t: Cow<'_, str> = Cow::Owned(String::from("plain"));
    println!("{} {}", marker(&s), marker(&t));
    let r: &str = &t;
    println!("{} {}", r, r.len());
    let v: Cow<'_, [u8]> = Cow::Owned(vec![7, 8]);
    println!("{}", first(&v));
    let refs = vec![&s, &t];
    for c in refs {
        println!("{}", marker(c));
    }
    println!("{}", captured());
}

#[allow(dead_code)]
fn captured() -> bool {
    let s: Cow<'_, str> = Cow::Borrowed("q@");
    let f = || marker(&s);
    f()
}
