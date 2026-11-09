// expect: L0733
// rustc: lifetime may not live long enough; the trait's bound output borrows `x`, not `'a`.
trait Pick<F: Fn(&u8) -> &u8> {
    fn pick<'a, 'b>(&self, f: &F, _a: &'a u8, x: &'b u8) -> &'a u8 {
        f(x)
    }
}
fn main() {}
