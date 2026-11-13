// expect: L0734
// rustc E0308: method not compatible with trait. The trait's `&self` takes
// any lifetime; `&'a self` fixes it to the impl's.
struct W<'a>(&'a str);
impl<'a> AsRef<str> for W<'a> {
    fn as_ref(&'a self) -> &'a str {
        self.0
    }
}
fn main() { println!("{}", W("x").as_ref()); }
