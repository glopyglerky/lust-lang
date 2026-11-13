// expect: L0643
// rustc E0034: core's blanket `Into` and the user's `into` both take the
// `&String` receiver by value at the first step: multiple applicable items.
trait Sh {
    fn into(&self) -> u8;
}
impl Sh for String {
    fn into(&self) -> u8 {
        1
    }
}
fn main() {
    let s = String::from("s");
    let r = &s;
    let v: u8 = r.into();
    println!("{}", v);
}
