// expect: L0612
// Valid Rust: fat raw pointers hash address and metadata; lust gives them no Hash impl (tf-fatptr).
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
    let mut h = std::collections::HashSet::new();
    h.insert(p);
    println!("{}", h.len());
}
