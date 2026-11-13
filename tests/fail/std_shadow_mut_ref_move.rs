// expect: L0305
// rustc E0507: through `x: &mut Vec<u8>`, the `Vec<u8>` step's by-value
// probe finds the trait's `is_empty(self)` before Vec's `&self` one (whose
// receiver `&Vec<u8>` is no by-value receiver at the `&mut` step), and the
// call cannot move out of `*x`.
trait Tr {
    fn is_empty(self) -> u32;
}
impl Tr for Vec<u8> {
    fn is_empty(self) -> u32 {
        4242
    }
}
fn main() {
    let mut v0 = vec![1u8, 2];
    let x = &mut v0;
    println!("{:?}", x.is_empty());
}
