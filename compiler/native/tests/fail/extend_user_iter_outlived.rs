// expect: L0306
// rustc E0597: the set keeps the `&str` items, which borrow `text`.
use std::collections::HashSet;
struct Words<'a>(&'a str);
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let s: &'a str = self.0;
        let s = s.trim_start();
        if s.is_empty() { return None; }
        let end = s.as_bytes().iter().position(|b| *b == b' ').unwrap_or(s.len());
        self.0 = &s[end..];
        Some(&s[..end])
    }
}
fn main() {
    let mut s: HashSet<&str> = HashSet::new();
    let mut v: Vec<&str> = Vec::new();
    {
        let text = String::from("a bb");
        s.extend(Words(&text));

    }
    println!("{s:?} {v:?}");
}
