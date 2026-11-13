// expect: L0110
// rustc E0005: a plain `let` name that is a block's const is the const's
// pattern, which can fail.
#![allow(non_snake_case)]
fn main() {
    const X: u32 = 7;
    {
        let X = 1u32;
        println!("{}", X);
    }
}
