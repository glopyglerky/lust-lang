// expect: L0110
// rustc E0530: a nested fn's parameter cannot shadow a static of the
// enclosing block.
#![allow(non_snake_case)]
fn main() {
    static X: u32 = 7;
    fn g(X: u32) -> u32 {
        X
    }
    println!("{}", g(1));
}
