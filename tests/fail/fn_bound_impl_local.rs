// expect: L0306
// rustc E0597: the impl's bound `F: Fn(&'a u8)` needs an argument that lives for `'a`.
struct W<'a, F> {
    f: F,
    r: &'a u8,
}
impl<'a, F: Fn(&'a u8)> W<'a, F> {
    fn go(&self) {
        let x = 5u8;
        (self.f)(&x);
    }
}
fn main() {}
