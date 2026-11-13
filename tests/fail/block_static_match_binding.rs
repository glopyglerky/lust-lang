// expect: L0110
// rustc E0530: a match binding cannot shadow a block's static.
#![allow(non_snake_case)]
fn main() {
    static X: u32 = 7;
    match 1u32 {
        X => println!("{}", X),
    }
}
