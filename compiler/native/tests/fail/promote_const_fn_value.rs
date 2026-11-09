// expect: L0306
// rustc E0716: a const fn result qualifies by its type (Option<String> needs drop), so `&C` is not promoted.
const fn mk() -> Option<String> {
    None
}
const C: Option<String> = mk();
fn main() {
    let r: &'static Option<String> = &C;
    println!("{}", r.is_none());
}
