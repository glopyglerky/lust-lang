// expect: L0306
// rustc E0597: a user method's result through the Cow's view borrows
// the Cow, which dies first.
use std::borrow::Cow;

#[derive(Clone)]
struct D {
    s: String,
}
impl D {
    fn get(&self) -> &str {
        &self.s
    }
}

fn main() {
    let r: &str;
    {
        let c: Cow<D> = Cow::Owned(D { s: "x".to_string() });
        r = c.get();
    }
    println!("{}", r);
}
