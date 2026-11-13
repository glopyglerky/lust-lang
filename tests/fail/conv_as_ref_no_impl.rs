// expect: L0612
// rustc E0277: String is not AsRef<i32>.
fn main() {
    let s = String::from("a");
    let r: &i32 = s.as_ref();
    println!("{}", r);
}
