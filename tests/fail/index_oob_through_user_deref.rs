// expect: L0541
// rustc: unconditional_panic follows a user Deref to an array.
struct W([u8; 2]);
impl std::ops::Deref for W {
    type Target = [u8; 2];
    fn deref(&self) -> &[u8; 2] {
        &self.0
    }
}
fn main() {
    let w = W([1, 2]);
    println!("{}", w[1]);
    println!("{}", w[2]);
}
