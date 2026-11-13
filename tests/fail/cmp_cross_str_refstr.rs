// expect: L0612
// rustc E0277: `str: PartialEq<&str>` has no impl (only `&str` with `&str`).
fn main() {
    let a = "a";
    println!("{}", a.eq(&"b"));
}
