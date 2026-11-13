// expect: L0112
// rustc E0277: a 13-tuple field is not PartialEq, so PartialEq cannot be derived.
#[derive(PartialEq)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}
fn main() {
    let s = S { t: (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8) };
    println!("{}", s == s);
}
