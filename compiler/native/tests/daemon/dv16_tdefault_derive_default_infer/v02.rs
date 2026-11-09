// A trait default body builds an S with Default::default(); Default goes out (cold L0612) and back.
const ROUTE: u8 = 0;
#[derive(Clone, Copy)]
struct S {
    x: u32,
}
trait Tr { fn g(&self) -> u32 { let s: S = Default::default(); s.x } } impl Tr for u8 {}
fn main() {
    let _r = ROUTE;
    println!("{}", 1u8.g());
}
