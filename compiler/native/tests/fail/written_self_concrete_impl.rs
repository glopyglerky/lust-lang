// expect: L0110
// rustc E0599: `make` is in `impl S<u8>`; `S<u16>` has no such fn.
struct S<T>(T);
impl S<u8> {
    fn make(t: u8) -> Self {
        S(t)
    }
}
fn main() {
    let a = S::<u16>::make(5);
    println!("{}", a.0);
}
