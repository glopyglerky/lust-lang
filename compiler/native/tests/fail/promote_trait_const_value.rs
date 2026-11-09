// expect: L0306
// rustc E0716: rustc does not look inside a trait associated const, so its type (it needs drop) decides.
trait Tr {
    const N: Option<String>;
}
impl Tr for u8 {
    const N: Option<String> = None;
}
fn main() {
    let r: &'static Option<String> = &<u8 as Tr>::N;
    println!("{}", r.is_none());
}
