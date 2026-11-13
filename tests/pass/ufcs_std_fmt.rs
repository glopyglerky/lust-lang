// UFCS calls of std fmt trait methods (ts-rust core.rs `impl Debug for
// Name`): `Debug::fmt(s, f)`, `<str as Display>::fmt(s, f)`, at builtin
// Self types, slices through Deref, and a derived Debug reached from a
// generic impl. The caller's Formatter flags (width, alternate) apply.
use std::fmt;
use std::ops::Deref;

struct Name(String);
impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        std::fmt::Debug::fmt(self.0.as_str(), f)
    }
}
impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt(&self.0, f)
    }
}

#[derive(Debug, Clone, Copy)]
struct Node(u32);
struct Decls(Vec<Node>);
impl Deref for Decls {
    type Target = [Node];
    fn deref(&self) -> &[Node] {
        &self.0
    }
}
impl fmt::Debug for Decls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

struct Wrap<T>(T);
impl<T: fmt::Debug> fmt::Debug for Wrap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

struct Id(u64);
impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)?;
        <char as fmt::Debug>::fmt(&'!', f)
    }
}

fn main() {
    let n = Name("a\"b".to_string());
    println!("{n:?} [{n:>6}] [{n:<8?}] {n}");
    let d = Decls(vec![Node(1), Node(2)]);
    println!("{d:?}");
    println!("{d:#?}");
    println!("{:?} {:?} [{:>6?}]", Wrap(Node(9)), Wrap("s"), Wrap(1.5));
    println!("{} [{:>5}] {:+}", Id(7), Id(8), Id(9));
}
