// expect: L0641
// rustc E0277: cast::<U>() needs a sized U (tf-fatptr).
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
    let q = p.cast::<str>();
    println!("{}", q.is_null());
}
