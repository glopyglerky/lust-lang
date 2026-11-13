// A `collect` target that only a later use fixes (tsrust T11 tspath
// `get_common_parents`: the tuple it returns).
use std::collections::HashSet;
fn parents(v: &[String]) -> (Vec<String>, usize) {
    let r = v.iter().map(|s| s.to_uppercase()).collect();
    (r, v.len())
}
fn joined(v: &[String]) -> String {
    let s = v.iter().map(|s| s.as_str()).collect();
    s
}
fn set(v: &[String]) -> usize {
    let h = v.iter().cloned().collect();
    let h: HashSet<String> = h;
    h.len()
}
fn main() {
    let v = vec![String::from("a"), String::from("b"), String::from("a")];
    println!("{:?} {} {}", parents(&v), joined(&v), set(&v));
}
