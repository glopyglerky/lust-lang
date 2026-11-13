// expect: L0612
// rustc E0277: a 13-tuple does not satisfy a `T: Eq` bound.
fn e<T: Eq>(x: T) -> T {
    x
}
fn main() {
    let _ = e((0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8));
    println!("ok");
}
