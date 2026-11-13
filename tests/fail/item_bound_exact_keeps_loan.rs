// expect: L0311
// rustc E0505: through `I: Iterator<Item = T>` the result is the
// iterator's item, a borrow of `v`, so `v` cannot move while it lives.
fn first<T, I: Iterator<Item = T>>(mut i: I) -> T {
    i.next().unwrap()
}
fn main() {
    let v = vec![String::from("a")];
    let r = first(v.iter().map(|s| s.as_str()));
    let w = v;
    println!("{} {}", r, w.len());
}
