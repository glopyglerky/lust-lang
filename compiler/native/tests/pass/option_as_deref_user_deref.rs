// A user Deref type: core's generic `impl<T: Deref> Option<T>` calls
// the user's `deref`.
use std::ops::Deref;
struct W(String);
impl Deref for W {
    type Target = String;
    fn deref(&self) -> &String {
        &self.0
    }
}
fn main() {
    let o = Some(W(String::from("x")));
    println!("{:?}", o.as_deref());
}
