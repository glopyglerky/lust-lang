// expect: L0307
// rustc E0515: `Self::Item` is `&'a str`; the body returns a borrow of a local.
struct Words<'a>(&'a str, String);
impl<'a> Iterator for Words<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        let local = String::from("x");
        Some(local.as_str())
    }
}
fn main() {
    let mut w = Words("ab", String::new());
    println!("{:?}", w.next());
}
