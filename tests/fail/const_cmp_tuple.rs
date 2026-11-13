// expect: L0630
// rustc E0658: tuple `==` is not a const operation (no const PartialEq).
const X: bool = (1, 2) == (1, 2);
fn main() {
    println!("{}", X);
}
