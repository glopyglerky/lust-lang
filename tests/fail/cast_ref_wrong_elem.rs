// expect: L0116
// rustc E0605: no coercion from `&[u8; 2]` to `&[u16]`, so no cast.
fn main() {
    let a = [1u8, 2];
    let s = &a as &[u16];
    println!("{}", s.len());
}
