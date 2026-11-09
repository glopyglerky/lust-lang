// expect: L0110
// rustc E0282: `G::N` leaves the impl's `T` unknown.
struct G<T>(T);
impl<T> G<T> {
    const N: usize = 3;
}
fn f() -> [u8; G::N] {
    [7; 3]
}
fn main() {
    println!("{}", f().len());
}
