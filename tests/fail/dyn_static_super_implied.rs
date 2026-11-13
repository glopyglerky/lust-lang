// expect: L0306
// rustc E0597: the `'static` bound comes from the supertrait `Root`.
trait Root: 'static {}
trait Read: Root {
    fn read(&self) -> u32;
}
struct W<X> {
    x: X,
}
impl<X: 'static> Root for W<X> {}
impl<X: 'static> Read for W<X> {
    fn read(&self) -> u32 {
        4
    }
}
fn main() {
    let n = 7u32;
    let w = W { x: &n };
    let d: &dyn Read = &w;
    println!("{}", d.read());
}
