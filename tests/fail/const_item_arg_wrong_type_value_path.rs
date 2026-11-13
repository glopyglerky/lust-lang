// expect: L0110
// rustc E0308: `K` is a u16, the param is a u8 (lust skipped the check on value paths).
struct S<const N: u8>;
impl<const N: u8> S<N> {
    fn n(&self) -> u8 {
        N
    }
}
const K: u16 = 3;
fn main() {
    println!("{}", S::<K>.n());
}
