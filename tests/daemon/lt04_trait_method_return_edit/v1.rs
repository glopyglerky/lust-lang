// A trait method's return goes from `&'static str` to borrowing
// `self` (trait and impl); the caller must recheck (W-BORROW).
trait Name {
    fn name(&self) -> &'static str;
}

struct A {
    s: &'static str,
}

impl Name for A {
    fn name(&self) -> &'static str {
        self.s
    }
}

fn main() {
    let r;
    {
        let a = A { s: "a" };
        r = a.name();
    }
    println!("{}", r);
}
