// expect: L0610
// rustc E0283: String has several AsRef impls; the target is unknown.
fn main() {
    let s = String::from("a");
    let r = s.as_ref();
    let _ = r;
}
