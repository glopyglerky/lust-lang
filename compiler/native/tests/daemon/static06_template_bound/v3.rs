// A `'static` bound added to a callee makes an unchanged generic body
// fail its template check (rustc E0310): a warm daemon must not reuse
// the stored template check of `h` across that edit.
fn need<Y: 'static>(_y: Y) -> u32 {
    7
}
fn h<X: 'static>(x: X) -> u32 {
    need(x) + 1
}
fn main() {
    println!("{}", h(1u8));
}
