// expect: L0306
// rustc E0597: the item `find` returns borrows `text`, not the iterator.
struct Words<'a>(&'a str);
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let s: &'a str = self.0;
        if s.is_empty() {
            return None;
        }
        self.0 = &s[1..];
        Some(&s[..1])
    }
}
fn main() {
    let r;
    {
        let text = String::from("abc");
        let mut w = Words(&text);
        r = w.find(|s| s.len() > 0);
    }
    println!("{r:?}");
}
