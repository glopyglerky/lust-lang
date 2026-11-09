// expect: L0110
// rustc E0308: the const item is a `u8`, the parameter a `usize`.
const K: u8 = 8;
struct S;
impl S {
    fn m<const P: usize>(&self) -> usize {
        P
    }
}
fn main() {
    println!("{}", S.m::<K>());
}
