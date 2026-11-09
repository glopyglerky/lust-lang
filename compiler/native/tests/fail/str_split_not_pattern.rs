// expect: L0612
// rustc E0277: an integer is not a str pattern.
fn main() {
    println!("{:?}", "a5b".split(5).count());
}
