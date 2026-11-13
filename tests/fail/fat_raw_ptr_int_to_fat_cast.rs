// expect: L0116
// rustc E0606: an integer does not cast to a fat raw pointer (tf-fatptr).
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
    let q = 0usize as *const str;
    println!("{}", q.is_null());
}
