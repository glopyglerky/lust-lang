// expect: L0733
// rustc: lifetime may not live long enough. The outer reference `get`
// returns borrows the set; only the element inside is `'static`.
use std::collections::HashSet;
fn g<'s>(s: &'s HashSet<&'static str>) -> &'static &'static str {
    s.get("a").unwrap()
}
fn main() {
    let mut s = HashSet::new();
    s.insert("a");
    println!("{}", g(&s));
}
