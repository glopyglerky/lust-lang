// expect: L0602
// Valid Rust: with_addr keeps a fat pointer metadata; lust does not build it (tf-fatptr).
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
    let v = [1u32, 2];
    let p: *const [u32] = &v[..];
    let q = p.with_addr(p.addr());
    println!("{}", q.len());
}
