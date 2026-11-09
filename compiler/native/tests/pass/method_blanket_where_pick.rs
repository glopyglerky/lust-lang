// Two traits with one method name, each with a blanket impl: only the
// impl whose where clause holds is a candidate, so the call is not
// ambiguous.
trait X {}
trait Y {}
trait P {
    fn f(&self) -> &'static str;
}
trait Q {
    fn f(&self) -> &'static str;
}
impl<T: X> P for T {
    fn f(&self) -> &'static str {
        "P"
    }
}
impl<T: Y> Q for T {
    fn f(&self) -> &'static str {
        "Q"
    }
}
struct S;
impl X for S {}
struct U;
impl Y for U {}
fn main() {
    println!("{} {}", S.f(), U.f());
}
