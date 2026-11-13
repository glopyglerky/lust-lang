// expect: L0115
// rustc E0004: a block's const in a pattern matches only its value, so
// the match is not exhaustive.
#![allow(non_snake_case)]
fn main() {
    const X: u32 = 7;
    const T: bool = true;
    match 5u32 {
        X => println!("seven"),
    }
    match false {
        T => println!("t"),
    }
}
