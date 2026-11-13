// expect: L0112
// Valid Rust: fat raw pointers order by address and metadata; lust does not build it (tf-fatptr).
trait Tr {
    fn f(&self) -> u32;
}
struct S(u32);
impl Tr for S {
    fn f(&self) -> u32 {
        self.0
    }
}
fn main() {
    let v = [1u8, 2];
    let p: *const [u8] = &v[..];
    let q: *const [u8] = &v[1..];
    println!("{}", p < q);
}
