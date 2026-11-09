// expect: L0203
// rustc E0401: a static inside a fn cannot name the fn's lifetime.
fn f<'a>(x: &'a u8) -> u8 {
    static S: Option<&'a u8> = None;
    let _ = S;
    *x
}
fn main() {
    f(&1);
}
