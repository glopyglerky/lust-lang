// `fmt` called as a method on a `&str` receiver (`s.as_str().fmt(f)`):
// the receiver is the fat `&str` value itself, not a pointer to it.
use std::fmt;

struct W(String);
impl fmt::Debug for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_str().fmt(f)
    }
}
impl fmt::Display for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_str().fmt(f)
    }
}

struct S<'a>(&'a [u8]);
impl fmt::Debug for S<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

struct B(Box<str>);
impl fmt::Debug for B {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (*self.0).fmt(f)?;
        self.0.fmt(f)
    }
}

struct T(&'static str, u32);
impl fmt::Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "lit".fmt(f)?;
        self.0.fmt(f)?;
        (*self.0).fmt(f)?;
        self.1.fmt(f)
    }
}

fn main() {
    println!("{:?} {} {:>6}|", W("a\"b".to_string()), W("x".to_string()), W("p".to_string()));
    println!("{:?}", S(&[1, 2]));
    println!("{:?}", B("bx".into()));
    println!("{} {:>8}|", T("s", 3), T("q", 4));
}
