// expect: L0104
// rustc E0277: no From<&str> for Vec<u16>.
fn main() {
    let v: Vec<u16> = "ab".into();
    println!("{v:?}");
}
