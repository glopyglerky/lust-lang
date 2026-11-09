// Through `x: &mut Vec<u8>`, the `&mut Vec<u8>` step's by-value probe
// takes neither Vec's `&self` is_empty nor the trait's by-value one; the
// `Vec<u8>` step's by-value probe then finds the trait first. Here the
// trait is implemented for `&mut Vec<u8>`, so rustc calls it at step 0.
trait Tr {
    fn is_empty(self) -> u32;
}
impl Tr for &mut Vec<u8> {
    fn is_empty(self) -> u32 {
        self.push(9);
        self.len() as u32
    }
}
fn main() {
    let mut v0 = vec![1u8, 2];
    let x = &mut v0;
    println!("{}", x.is_empty());
    let y = &v0;
    println!("{} {:?}", y.is_empty(), v0);
}
