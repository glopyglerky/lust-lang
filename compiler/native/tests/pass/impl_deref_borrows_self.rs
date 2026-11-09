// A user `Deref` result borrows `self` for as long as it is used.
use std::ops::Deref;
struct W { s: String }
impl Deref for W { type Target = String; fn deref(&self) -> &String { &self.s } }
fn main() {
    let w = W { s: String::from("w") };
    let r: &String = w.deref();
    let n = r.len();
    let q = w.deref().as_str();
    println!("{} {} {}", r, n, q);
}
