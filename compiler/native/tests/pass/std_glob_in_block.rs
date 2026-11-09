// A block's std glob shadows the crate-root `min`: rustc prints 1.
fn min(a: i32, b: i32) -> i32 { a * 100 + b }
fn main() {
    use std::cmp::*;
    println!("{}", min(1, 2));
}
