// expect: L0306
// rustc E0597: `Self::V(..)` in `impl<'a> E<'a>` has the impl's `'a`, which a local cannot outlive.
struct D(&'static str);
enum E<'a> {
    V(&'a D),
    N,
}
impl<'a> E<'a> {
    fn go() {
        let d = D("d");
        let _x = Self::V(&d);
        println!("in");
    }
}
fn main() {
    E::go();
}
