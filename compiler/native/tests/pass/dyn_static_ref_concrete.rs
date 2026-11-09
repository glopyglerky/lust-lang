// ARCH 2.8: `dyn Tr + 'static` over a type that holds a reference. The reference is 'static
// here, so rustc accepts; the borrow checker proves it at the unsizing (R02).
trait Tr {
    fn get(&self) -> u8;
}
impl Tr for &u8 {
    fn get(&self) -> u8 {
        **self
    }
}
static X: u8 = 5;
fn keep(d: &(dyn Tr + 'static)) -> u8 {
    d.get()
}
fn main() {
    let r = &X;
    println!("{}", keep(&r));
}
