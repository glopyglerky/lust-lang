// expect: L0733
// rustc: lifetime may not live long enough. The projection is `&'static u8`.
pub trait Tr {
    type Out;
}
impl Tr for u8 {
    type Out = &'static u8;
}
fn main() {
    let f: fn(&u8) -> <u8 as Tr>::Out = |a| a;
    let y = 2u8;
    println!("{}", f(&y));
}
