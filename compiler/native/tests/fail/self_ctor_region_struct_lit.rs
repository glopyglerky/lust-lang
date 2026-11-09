// expect: L0306
// rustc E0716: `Self { .. }` in `impl<'a> H<'a>` has the impl's `'a`, so a temporary cannot fill it.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct H<'a> {
    r: &'a D,
}
impl<'a> H<'a> {
    fn go() {
        let _h = Self { r: &D("t") };
        println!("in");
    }
}
fn main() {
    H::go();
}
