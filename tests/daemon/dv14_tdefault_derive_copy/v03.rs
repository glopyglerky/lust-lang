// A trait default body copies an S twice; Copy goes out (cold L0301) and back.
const ROUTE: u8 = 0;
#[derive(Clone, Copy, Debug)]
struct S {
    x: u32,
}
trait Tr { fn g(&self, s: S) -> u32 { let a = s; let b = s; a.x + b.x } } impl Tr for u8 {}
fn main() {
    let _r = ROUTE;
    println!("{}", 1u8.g(S { x: 1 }));
}
