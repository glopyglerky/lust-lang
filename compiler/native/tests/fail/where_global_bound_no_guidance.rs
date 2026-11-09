// expect: L0111
// rustc E0283: a global where-bound does not pick between two impls
// (rustc's winnowing prefers impls over global bounds).
trait Tr<X> {
    fn go(&self) -> X;
}
struct A;
impl Tr<u8> for A {
    fn go(&self) -> u8 {
        1
    }
}
impl Tr<u16> for A {
    fn go(&self) -> u16 {
        2
    }
}
fn needs()
where
    A: Tr<u8>,
{
    let x = A.go();
    println!("{}", x);
}
fn main() {
    needs();
}
