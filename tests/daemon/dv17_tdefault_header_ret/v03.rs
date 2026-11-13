// A trait default body returns h(); h's return type changes u32 -> u64 (cold L0110) and back.
const ROUTE: u8 = 0;
fn h() -> u32 { 1 }
trait Tr {
    fn g(&self) -> u32 {
        h()
    }
}
impl Tr for u8 {}
fn main() {
    let _r = ROUTE;
    println!("{}", 1u8.g());
}
