// An `Iterator` impl's items (`&'a str`) outlive the iterator.
struct Words<'a> { s: &'a str }
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.s.is_empty() { return None; }
        let w = self.s;
        self.s = "";
        Some(w)
    }
}
fn main() {
    let text = String::from("hello");
    let first;
    {
        let mut w = Words { s: &text };
        first = w.next();
    }
    println!("{:?}", first);
}
