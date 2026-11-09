// Elision with `self`: only a reference whose referent holds `Self` is an
// input lifetime (`&self`, `self: &Box<Self>`); the lifetimes inside a
// by-value `self` are not, so another parameter's lifetime is the only one.
struct W<'a> {
    r: &'a u8,
}
impl<'a> W<'a> {
    fn pick(self, x: &u8) -> &u8 {
        x
    }
    fn get(self: &Self) -> &u8 {
        self.r
    }
    fn get_box(self: &Box<Self>) -> &u8 {
        self.r
    }
    fn mine(&mut self, _x: &u8) -> &u8 {
        self.r
    }
    fn new(r: &u8) -> W<'_> {
        W { r }
    }
}
fn take(w: W) -> &u8 {
    w.r
}
fn main() {
    let v = 1u8;
    let x = 2u8;
    let mut w = W::new(&v);
    println!("{}", w.get());
    println!("{}", w.mine(&x));
    println!("{} {}", Box::new(W { r: &v }).get_box(), take(W { r: &x }));
    println!("{}", W { r: &v }.pick(&x));
}
