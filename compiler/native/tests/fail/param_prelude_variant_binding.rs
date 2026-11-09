// expect: L0110
// rustc E0530: function parameters cannot shadow tuple variants.
fn g(Ok: i32) -> i32 { Ok }
fn main() { println!("{}", g(2)); }
