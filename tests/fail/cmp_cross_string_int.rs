// expect: L0612
// rustc E0277: no `String: PartialEq<{integer}>` impl.
fn main() {
    let s = String::from("a");
    println!("{}", s.eq(&5));
}
