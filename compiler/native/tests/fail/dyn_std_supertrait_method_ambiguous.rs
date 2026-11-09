// expect: L0720
// rustc E0034: Debug is a supertrait of U, so Debug::fmt is an object
// candidate of `dyn U` (no import needed) and ties with U::fmt.
trait U: std::fmt::Debug {
    fn fmt(&self) -> u64;
}
#[derive(Debug)]
struct S;
impl U for S {
    fn fmt(&self) -> u64 {
        7
    }
}
fn main() {
    let x: Box<dyn U> = Box::new(S);
    println!("{}", x.fmt());
}
