// expect: L0612
// rustc E0271: HashSet<String> has no Extend<&String> (it needs T: Copy).
use std::collections::HashSet;

fn main() {
    let v = vec![String::from("a")];
    let mut s: HashSet<String> = HashSet::new();
    s.extend(&v);
    println!("{}", s.len());
}
