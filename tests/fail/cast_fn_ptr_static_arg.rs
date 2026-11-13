// expect: L0306
// rustc E0597: the cast fixes the pointer's input to `&'static u8`, so `x` must live for `'static`.
fn keep(_: &'static u8) {}
fn main() {
    let p = keep as fn(&'static u8);
    let x = 5u8;
    p(&x);
}
