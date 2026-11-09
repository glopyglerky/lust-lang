// expect: L0110
// rustc E0308: `dyn Tr` is not `Send`, so dropping the principal cannot give `dyn Send`.
trait Tr {}
impl Tr for u8 {}
fn main() {
    let a: Box<dyn Tr> = Box::new(1u8);
    let b: Box<dyn Send> = a;
    drop(b);
}
