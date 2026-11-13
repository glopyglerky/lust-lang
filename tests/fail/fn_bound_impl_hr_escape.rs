// expect: L0307
// rustc E0515: a higher-ranked bound's output borrows the call's argument, here a local.
struct H<F> {
    f: F,
}
impl<F: Fn(&u8) -> &u8> H<F> {
    fn go(&self) -> &u8 {
        let x = 5u8;
        (self.f)(&x)
    }
}
fn main() {}
