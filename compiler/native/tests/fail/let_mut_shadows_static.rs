// expect: L0110
// rustc E0530: `mut` makes the name a fresh binding, which may not
// shadow a static.
#![allow(non_snake_case)]
static X: u32 = 7;
fn main() {
    let mut X = 1u32;
    X += 1;
    println!("{}", X);
}
