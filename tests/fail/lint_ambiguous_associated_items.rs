// expect: L0622
// rustc denies `Self::V` (ambiguous_associated_items): the enum has a
// variant `V` and the trait an associated type `V`.
enum E {
    V,
    W,
}
trait Tr {
    type V;
    fn f(&self) -> Vec<Self::V>;
}
impl Tr for E {
    type V = u8;
    fn f(&self) -> Vec<Self::V> {
        Vec::new()
    }
}
fn main() {
    let _ = E::V;
    println!("{}", E::W.f().len());
}
