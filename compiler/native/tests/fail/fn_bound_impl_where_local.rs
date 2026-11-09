// expect: L0306
// rustc E0597: an impl where-clause `F: FnMut(&'a u8)` needs an argument that lives for `'a`.
struct W<'a, F> {
    f: F,
    r: &'a u8,
}
impl<'a, F> W<'a, F>
where
    F: FnMut(&'a u8),
{
    fn go(&mut self) {
        let x = 5u8;
        (self.f)(&x);
    }
}
fn main() {}
