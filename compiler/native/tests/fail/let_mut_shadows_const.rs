// expect: L0110
// rustc E0530: `mut` makes the name a fresh binding, which may not
// shadow a const (a plain name would be the const's pattern).
#![allow(non_snake_case)]
fn main() {
    const X: u32 = 7;
    let mut X = 1u32;
    X += 1;
    println!("{}", X);
}
