// expect: L0104
// rustc E0277: no From<&&str> for String.
fn main() {
    let s: String = (&&"x").into();
    println!("{s}");
}
