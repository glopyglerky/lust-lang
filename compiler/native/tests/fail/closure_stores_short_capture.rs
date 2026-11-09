// expect: L0306
// The same store with a capture that borrows a local: the creator cannot
// prove the capture is `'static` (rustc E0597).
fn with<R>(f: impl FnOnce(&mut Vec<&'static str>) -> R) -> R {
    let mut v = Vec::new();
    f(&mut v)
}
fn main() {
    let x = String::from("x");
    let s: &str = &x;
    with(|d| d.push(s));
}
