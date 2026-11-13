// expect: L0306
// rustc E0597: the impl's `T: 'static` bound applies to `W::new`.
struct W<T> { t: T }
impl<T: 'static> W<T> { fn new(t: T) -> Self { W { t } } }
fn main() {
    let x = 5;
    let w = W::new(&x);
    println!("{}", w.t);
}
