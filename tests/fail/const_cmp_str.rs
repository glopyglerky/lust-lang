// expect: L0630
// rustc E0658: `&str` `==` is not a const operation.
const X: bool = "a" == "a";
fn main() {
    println!("{}", X);
}
