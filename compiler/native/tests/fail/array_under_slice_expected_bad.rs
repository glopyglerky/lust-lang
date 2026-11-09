// expect: L0612
// The elements coerce to the expected `Box<dyn Opt>`, and `u8` does not
// implement `Opt` (rustc E0277).
trait Opt {
    fn n(&self) -> u8;
}
fn watch(opts: &[Box<dyn Opt>]) -> usize {
    opts.len()
}
fn main() {
    println!("{}", watch(&[Box::new(5u8)]));
}
