// expect: L0612
// rustc E0277: a 13-tuple does not satisfy a `T: Default` bound.
fn d<T: Default>() -> T {
    T::default()
}
fn main() {
    let t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) = d();
    println!("{}", t.12);
}
