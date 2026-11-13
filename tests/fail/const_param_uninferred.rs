// expect: L0610
// rustc E0284: nothing fixes the const param.
struct B<const N: usize> {
    v: u8,
}
impl<const N: usize> B<N> {
    fn new() -> Self {
        B { v: N as u8 }
    }
}
fn main() {
    let b = B::new();
    println!("{}", b.v);
}
