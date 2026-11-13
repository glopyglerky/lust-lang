// expect: L0112
// Valid Rust: Debug of a fat raw pointer prints its address; lust does not build it (tf-fatptr).
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
    let v = [1u8];
    let p: *const [u8] = &v[..];
    println!("{}", format!("{:?}", p).starts_with("0x"));
}
