// `Self` in an impl with lifetimes names the impl's own lifetimes: values
// built through it may hold borrows that outlive them. The plain type
// name gets fresh lifetimes instead, so it may hold a local borrow.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct H<'a>(&'a D);
enum E<'a> {
    V(&'a D),
    N,
}
struct S<'a> {
    r: &'a D,
}
impl<'a> H<'a> {
    fn named() -> usize {
        let h = H(&D("t"));
        h.0.0.len()
    }
    fn wrap(r: &'a D) -> H<'a> {
        H(r)
    }
}
impl<'a> E<'a> {
    fn pick(r: &'a D, on: bool) -> Self {
        let mut x = Self::N;
        if on {
            x = Self::V(r);
        }
        x
    }
}
impl<'a> S<'a> {
    fn new(r: &'a D) -> Self {
        Self { r }
    }
    fn local() -> usize {
        let d = D("local");
        let s = S { r: &d };
        s.r.0.len()
    }
}
fn main() {
    let d = D("d");
    println!("{}", H::named());
    println!("{}", H::wrap(&d).0.0);
    if let E::V(r) = E::pick(&d, true) {
        println!("{}", r.0);
    }
    println!("{} {}", S::new(&d).r.0, S::local());
}
