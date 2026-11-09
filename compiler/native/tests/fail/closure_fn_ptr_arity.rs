// expect: L0105
// rustc E0593: the fn pointer takes one argument, the closure two.
fn main() {
    let p: fn(u8) -> u8 = |a, b| a + b;
    println!("{}", p(4));
}
