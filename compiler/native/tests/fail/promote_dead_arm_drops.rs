// expect: L0306
// rustc E0716: the const's qualifs join both arms, and the dead one holds a String, so `&C` is not promoted.
const C: Option<String> = if false { Some(String::new()) } else { None };
fn main() {
    let r: &'static Option<String> = &C;
    println!("{}", r.is_none());
}
