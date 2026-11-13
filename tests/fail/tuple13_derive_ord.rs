// expect: L0110
// rustc E0369/E0277: a 13-tuple field is not Eq or Ord, so Ord cannot be derived.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct S {
    t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8),
}
fn main() {
    let s = S { t: (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8) };
    println!("{:?}", s.cmp(&s));
}
