// expect: L0305
// rustc E0507: `r()` on `r: &impl FnOnce` calls the opaque by value, a move
// out of `*r`, even when the hidden closure is `Fn`.
fn wrap<T: Copy>(t: T) -> impl FnOnce() -> T { move || t }
fn main() {
    let h = wrap(7u8);
    let r = &h;
    let a = r();
    println!("{a}");
}
