// expect: L0643
// rustc E0061: `<Vec<u8>>::spare_capacity_mut` is Vec's inherent method
// (it takes `&mut self`), not the user trait's associated fn.
trait T {
    fn spare_capacity_mut() -> u8;
}
impl T for Vec<u8> {
    fn spare_capacity_mut() -> u8 {
        7
    }
}
type V = Vec<u8>;
fn main() {
    println!("{}", <Vec<u8>>::spare_capacity_mut());
    println!("{}", V::spare_capacity_mut());
}
