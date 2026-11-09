// `s.as_mut_str()` is String's inherent method, which wins over the
// user trait's method of that name (fx-option-rest). The trait is
// named with `Mut::as_mut_str(&mut s)`.
trait Mut {
    fn as_mut_str(&mut self) -> u8;
}
impl Mut for String {
    fn as_mut_str(&mut self) -> u8 {
        3
    }
}
fn main() {
    let mut s = String::from("ab");
    println!("{:?}", s.as_mut_str());
    println!("{}", Mut::as_mut_str(&mut s));
}
