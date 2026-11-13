// expect: L0612
// rustc E0277: a 13-tuple does not satisfy a `T: PartialEq` bound.
fn e<T: PartialEq>(a: &T) -> bool {
    a == a
}
fn main() {
    println!("{}", e(&(0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8)));
}
