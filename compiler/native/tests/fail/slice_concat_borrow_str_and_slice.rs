// expect: L0610
// rustc E0284: an element that is Borrow<str> and Borrow<[u8]> fits both of
// core's Concat impls, and concat has no separator to pick one.
use std::borrow::Borrow;
struct B(String);
impl Borrow<str> for B {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl Borrow<[u8]> for B {
    fn borrow(&self) -> &[u8] {
        self.0.as_bytes()
    }
}
fn main() {
    let v = vec![B("a".into()), B("b".into())];
    println!("{}", v.concat());
}
