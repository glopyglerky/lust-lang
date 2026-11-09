// A trait default body calls h(1); h's param becomes &u32 (cold L0110) and back.
const ROUTE: u8 = 0;
fn h(x: &u32) -> u32 { *x }
trait Tr {
    fn g(&self) -> u32 {
        h(1)
    }
}
impl Tr for u8 {}
fn main() {
    let _r = ROUTE;
    println!("{}", 1u8.g());
}
