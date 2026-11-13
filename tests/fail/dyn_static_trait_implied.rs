// expect: L0306
// rustc E0597: `trait T: 'static` makes `&dyn T` a `'static` object, so
// the borrow of `n` inside `W` must live for `'static`.
trait T: 'static {
    fn read(&self) -> u32;
}
struct W<X> {
    x: X,
}
impl<X: 'static> T for W<X> {
    fn read(&self) -> u32 {
        4
    }
}
fn main() {
    let n = 7u32;
    let w = W { x: &n };
    let d: &dyn T = &w;
    println!("{}", d.read());
}
