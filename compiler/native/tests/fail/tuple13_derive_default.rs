// expect: L0110
// rustc E0277: a 13-tuple field is not Default, so Default cannot be derived.
#[derive(Default, Debug)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}
fn main() {
    println!("{:?}", S::default());
}
