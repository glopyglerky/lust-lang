// expect: L0112
// rustc E0277: a 13-tuple field is not Debug, so Debug cannot be derived.
#[derive(Debug)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}
fn main() {
    println!("{:?}", S { t: (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8) });
}
