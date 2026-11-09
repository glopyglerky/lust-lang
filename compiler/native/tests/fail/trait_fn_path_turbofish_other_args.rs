// expect: L0104
// rustc E0599: `M<u8>` does not implement `Mk`; only `M<u16>` does.
struct M<T>(T);
trait Mk {
    fn mk() -> Self;
}
impl Mk for M<u16> {
    fn mk() -> Self {
        M(16)
    }
}
fn main() {
    let m: M<u16> = M::<u8>::mk();
    println!("{}", m.0);
}
