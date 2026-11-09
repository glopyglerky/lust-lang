// expect: L0510
// rustc E0616: the field of `TryFromIntError` is private to core.
fn main() {
    let e = u8::try_from(300i32).unwrap_err();
    let _ = e.0;
}
