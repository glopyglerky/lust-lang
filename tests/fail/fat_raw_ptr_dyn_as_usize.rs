// expect: L0116
// rustc E0606: a fat raw pointer does not cast to an integer (tf-fatptr).
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
    println!("{} {}", p as usize, d.f());
}
