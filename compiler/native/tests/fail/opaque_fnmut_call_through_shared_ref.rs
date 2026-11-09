// expect: L0308
// rustc E0596: `r()` on `r: &impl FnMut` borrows `*r` mutably behind `&`,
// even when the hidden closure is `Fn`.
fn wrap(t: u8) -> impl FnMut() -> u8 { move || t }
fn main() {
    let h = wrap(7);
    let r = &h;
    let a = r();
    println!("{a}");
}
