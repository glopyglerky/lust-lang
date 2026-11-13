// expect: L0306
// rustc E0597: a `dyn T + '_` parameter of a `'static` trait still needs a
// `'static` source; `W<&u32>` borrows a local.
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
fn call(d: &(dyn T + '_)) -> u32 {
    d.read()
}
fn main() {
    let n = 7u32;
    println!("{}", call(&W { x: &n }));
}
