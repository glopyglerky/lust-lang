// expect: L0733
// rustc: lifetime may not live long enough. The second slot of the
// closure's output is `'static` in the caller's return type, but the
// body puts the argument's borrow there.
fn f<'a>(s: &'a u8) -> Option<(&'a u8, &'static u8)> {
    Some(s).map(|st| (st, st))
}
fn main() {
    let x = 3u8;
    println!("{:?}", f(&x));
}
