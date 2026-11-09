// expect: L0116
// rustc E0607: a thin raw pointer does not cast to a fat one (tf-fatptr).
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
    let x = 5u8;
    let p = &x as *const u8;
    let q = p as *const [u8];
    println!("{}", q.is_null());
}
