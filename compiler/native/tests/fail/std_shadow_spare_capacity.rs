// expect: L0112
// rustc E0277: `v.spare_capacity_mut()` is Vec's inherent method, which
// wins over the user trait's method of that name; its `&mut
// [MaybeUninit<u8>]` is not Display. lust must not pick the trait.
trait T {
    fn spare_capacity_mut(&mut self) -> u8;
}
impl T for Vec<u8> {
    fn spare_capacity_mut(&mut self) -> u8 {
        7
    }
}
fn main() {
    let mut v: Vec<u8> = Vec::new();
    println!("{}", v.spare_capacity_mut());
}
