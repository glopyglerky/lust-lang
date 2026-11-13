// `impl Into<T>` and `R: Into<T>` params called with a tuple, a user
// type, and `&str`: each instance converts through the user `From`
// impl the template's bound names.
struct Name(String);
impl From<&str> for Name {
    fn from(s: &str) -> Self {
        Name(s.to_string())
    }
}
impl From<String> for Name {
    fn from(s: String) -> Self {
        Name(s)
    }
}
struct Range {
    a: u32,
    b: u32,
}
impl From<(u32, u32)> for Range {
    fn from(p: (u32, u32)) -> Self {
        Range { a: p.0, b: p.1 }
    }
}
struct W(u8);
impl From<W> for Range {
    fn from(w: W) -> Self {
        Range { a: 0, b: w.0 as u32 }
    }
}
fn new_symbol(name: impl Into<Name>) -> usize {
    let n: Name = name.into();
    n.0.len()
}
fn span(r: impl Into<Range>) -> u32 {
    let r = r.into();
    r.b - r.a
}
fn span2<R: Into<Range>>(r: R) -> u32 {
    let r = r.into();
    r.b - r.a
}
fn main() {
    println!("{} {}", new_symbol("abc"), new_symbol(String::from("de")));
    println!("{} {} {} {}", span((3, 10)), span(W(4)), span2((1, 2)), span2(W(9)));
}
