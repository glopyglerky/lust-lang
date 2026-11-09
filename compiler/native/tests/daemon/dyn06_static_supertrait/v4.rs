// A `'static` supertrait bound added and removed on one warm daemon, with
// `call` and `main` unchanged. The bound makes `&dyn Read` a `'static`
// object, so the borrow of `n` behind `W` must fail (rustc E0597).
trait Root {}
trait Read: Root + 'static {
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
fn call(d: &dyn Read) -> u32 {
    d.read()
}
fn main() {
    let n = 7u32;
    let w = W { x: &n };
    println!("{}", call(&w));
}
