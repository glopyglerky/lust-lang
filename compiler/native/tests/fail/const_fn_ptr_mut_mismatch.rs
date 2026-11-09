// expect: L0110
// rustc E0308: `fn(&u8)` is not `fn(&mut u8)`; only subtyping applies.
fn f(x: &u8) -> u8 {
    *x
}
const F: fn(&mut u8) -> u8 = f;
fn main() {
    let g = F;
    let _ = g;
}
