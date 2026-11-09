// An impl method's return goes from the impl's `'a` to an elided
// lifetime that borrows `self`; the caller must recheck (W-BORROW).
struct S<'a> {
    r: &'a String,
}

impl<'a> S<'a> {
    fn get(&self) -> &String {
        self.r
    }
}

fn main() {
    let x = String::from("x");
    let r;
    {
        let s = S { r: &x };
        r = s.get();
    }
    println!("{}", r);
}
