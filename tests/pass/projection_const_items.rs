// Const and static item types with a projection normalize through the
// impl when its trait ref is proved (collect evaluates these items
// before it normalizes signatures).
trait Tr {
    type Out;
}
struct A;
impl Tr for A {
    type Out = u8;
}
struct B;
impl Tr for B {
    type Out = (i16, i16);
}
const C: <A as Tr>::Out = 200;
static S: <A as Tr>::Out = 100;
const P: [<B as Tr>::Out; 2] = [(1, -1), (2, -2)];
fn main() {
    println!("{} {} {}", C.wrapping_add(100), S, P[1].0 * P[1].1);
}
