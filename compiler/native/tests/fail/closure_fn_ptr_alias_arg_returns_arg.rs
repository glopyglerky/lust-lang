// expect: L0733
// rustc: lifetime may not live long enough. `P<&'static u8>` is `fn(&u8) -> Option<&'static u8>`.
type P<T> = fn(&u8) -> Option<T>;
fn main() {
    let f: P<&'static u8> = |a| Some(a);
    let y = 2u8;
    println!("{:?}", f(&y));
}
