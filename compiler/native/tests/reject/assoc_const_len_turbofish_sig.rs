// expect: L0110
// A signature array length with written type args on an inherent
// const path needs the typed pick, which signatures do not have yet.
struct G<T>(T);
impl<T> G<T> {
    const N: usize = 3;
}
fn f() -> [u8; G::<i32>::N] {
    [7; 3]
}
fn main() {
    println!("{}", f().len());
}
