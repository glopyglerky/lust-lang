// expect: L0612
// rustc E0277: i32 is not AsRef<str>.
fn shout<S: AsRef<str>>(s: S) -> usize {
    s.as_ref().len()
}
fn main() {
    println!("{}", shout(5));
}
