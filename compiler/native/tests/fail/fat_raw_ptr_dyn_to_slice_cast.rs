// expect: L0602
// rustc E0606: a dyn pointer does not cast to a slice pointer; lust rejects every fat-to-fat cast but a mutability change (tf-fatptr).
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
    let d: &dyn Tr = &S(1);
    let p = d as *const dyn Tr as *const [u8];
    println!("{} {}", p.is_null(), d.f());
}
