// expect: L0306
// rustc E0597: `Deref::deref` borrows `self` in the trait; an impl
// returning `&'static str` does not change what callers see.
use std::ops::Deref;
struct Name { s: String }
impl Deref for Name {
    type Target = str;
    fn deref(&self) -> &'static str { "fixed" }
}
fn main() {
    let r;
    {
        let n = Name { s: String::from("n") };
        r = n.deref();
        let _ = &n.s;
    }
    println!("{r}");
}
