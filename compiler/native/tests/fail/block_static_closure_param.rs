// expect: L0110
// rustc E0530: a closure parameter cannot shadow a block's static.
#![allow(non_snake_case)]
fn main() {
    static X: u32 = 7;
    let f = |X: u32| X;
    println!("{}", f(1));
}
