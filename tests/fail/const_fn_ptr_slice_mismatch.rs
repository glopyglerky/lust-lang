// expect: L0110
// rustc E0308: a fn taking `&[u8]` is not a `fn(&[u8; 2])`.
fn f(x: &[u8]) -> u8 {
    x[0]
}
static F: fn(&[u8; 2]) -> u8 = f;
fn main() {
    let g = F;
    let _ = g;
}
