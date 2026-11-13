// expect: L0612
// rustc E0277: there is no `From<&mut String>` for `String`.
fn main() {
    let mut s = String::from("a");
    let t = String::from(&mut s);
    println!("{}", t);
}
