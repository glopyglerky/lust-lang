// expect: L0104
// rustc E0599: `Self` is `M<u8>` here, and only `M<u16>` implements `Mk`.
struct M<T>(T);
trait Mk {
    fn mk() -> Self;
}
impl Mk for M<u16> {
    fn mk() -> Self {
        M(16)
    }
}
impl M<u8> {
    fn bad() -> M<u16> {
        Self::mk()
    }
}
fn main() {
    println!("{}", M::<u8>::bad().0);
}
