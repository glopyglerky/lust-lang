const fn cf() -> i32 { "bad" }
struct P;
impl P {
    const A: i32 = cf();
    const B: i32 = Self::A + 1;
}
trait Tr {
    const X: i32;
    const Y: i32 = Self::X * 3;
}
impl Tr for P {
    const X: i32 = P::B + 1;
}
const D: i32 = <P as Tr>::Y + 1;
fn main() {
    println!("{} {} {} {} {}", P::A, P::B, <P as Tr>::X, <P as Tr>::Y, D);
}
