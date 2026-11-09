// expect: L0720
// rustc E0034: std's `ToString::to_string` and the user trait's method
// both apply at the same probe of `String`.
trait Show {
    fn to_string(&self) -> u8;
}
impl Show for String {
    fn to_string(&self) -> u8 {
        1
    }
}
fn main() {
    let s = String::from("a");
    println!("{}", s.to_string());
}
