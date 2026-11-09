// expect: L0104
// rustc E0277: core's blanket `Into` is picked at the first autoderef step,
// so `W<String>` needs `String: From<W<String>>`; the deref target does not count.
use std::ops::Deref;
struct W<T>(T);
impl<T> Deref for W<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
fn main() {
    let v = W(String::from("s"));
    let o: String = v.into();
    println!("{:?}", o);
}
