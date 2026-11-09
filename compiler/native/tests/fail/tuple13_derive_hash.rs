// expect: L0110
// rustc E0277: a 13-tuple field is not Hash, so Hash cannot be derived.
#[derive(Hash)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}
fn main() {
    println!("ok");
}
