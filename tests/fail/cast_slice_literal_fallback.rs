// expect: L0116
// rustc E0605: casts are checked after literal fallback, so the array is
// `[i32; 2]` and `&[i32; 2] as &[u8]` is not a coercion.
fn main() {
    let a = &[1, 2] as &[u8];
    println!("{:?}", a);
}
