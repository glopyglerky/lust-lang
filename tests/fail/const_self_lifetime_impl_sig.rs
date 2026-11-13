// expect: L0110
// rustc: generic `Self` types are currently not permitted in anonymous
// constants. A lifetime parameter makes `Self` generic too.
struct C<'a>(&'a u8);
impl<'a> C<'a> {
    const N: usize = 3;
    fn f(&self) -> [u8; Self::N] {
        [1; 3]
    }
}
fn main() {
    let x = 1;
    println!("{}", C(&x).f().len());
}
