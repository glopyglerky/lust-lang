// Calls of `impl Fn*` values through references that rustc accepts: `Fn`
// through `&`, `FnMut` through `&mut`, and a `Copy` `FnOnce` copied out of `&`.
fn fn_wrap(t: u8) -> impl Fn() -> u8 { move || t }
fn mut_wrap(mut n: i32) -> impl FnMut() -> i32 { move || { n += 1; n } }
fn once_copy(t: u16) -> impl FnOnce() -> u16 + Copy { move || t + 1 }
fn main() {
    let f = fn_wrap(3);
    let r = &f;
    println!("{} {}", r(), (&&f)());
    let mut m = mut_wrap(10);
    let rm = &mut m;
    println!("{} {}", rm(), rm());
    let o = once_copy(4);
    let ro = &o;
    println!("{} {}", ro(), ro());
}
