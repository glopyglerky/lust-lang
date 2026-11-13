// expect: L0202
// rustc E0637: an impl's associated type cannot elide a lifetime.
trait T {
    type Out;
}
struct W<'a>(&'a u8);
impl<'a> T for W<'a> {
    type Out = Option<&'_ u8>;
}
fn main() {
    let _ = W(&1).0;
}
