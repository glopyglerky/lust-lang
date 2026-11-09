// expect: L0612
// rustc E0310: the impl of a `'static` trait needs `X: 'static`; the param
// sits 34 `Option`s deep in the header.
trait Root: 'static {}
struct Wrap<X> {
    x: X,
}
impl<X> Root for Wrap<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<Option<X>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> {}
fn main() {}
