// expect: L0001
// rustc: invalid reference to positional argument 1 (there is 1 argument).
fn main() {
    let a = 3;
    println!("{a} {1}", 1);
}
