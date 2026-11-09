// A trait default body formats an S with {:?}; Debug goes out (cold L0112) and back.
const ROUTE: u8 = 0;
#[derive(Clone, Copy, Debug)]
struct S {
    x: u32,
}
trait Tr { fn g(&self, s: S) -> String { format!("{:?}", s) } } impl Tr for u8 {}
fn main() {
    let _r = ROUTE;
    println!("{}", 1u8.g(S { x: 1 }));
}
