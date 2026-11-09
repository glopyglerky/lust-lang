// expect: L0110
// rustc E0599: no `N` for `G<i32>`. The signature scan must not
// read `G<u8>`'s const for it.
struct G<T>(T);
impl G<u8> {
    const N: usize = 3;
}
fn f() -> [u8; G::<i32>::N] {
    [7; 3]
}
fn main() {
    println!("{}", f().len());
}
