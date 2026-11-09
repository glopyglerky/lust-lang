// expect: L0104
// rustc E0599: only a slice pointer has len (tf-fatptr).
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
    let p = d as *const dyn Tr;
    println!("{} {}", p.len(), d.f());
}
