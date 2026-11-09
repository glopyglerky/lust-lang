// expect: L0110
#![allow(non_upper_case_globals)]
// rustc E0308: `Vec::<u8>::len` is Vec's inherent method (a fn item), which
// wins over the user trait's associated const of that name; lust resolves it
// to the method too, so the `&str` annotation is the same mismatch.
trait Named {
    const len: &'static str;
}
impl Named for Vec<u8> {
    const len: &'static str = "len";
}
fn main() {
    let s: &str = Vec::<u8>::len;
    println!("{}", s);
}
