// expect: L0735
// rustc rejects: `F: FnMut(&str)` makes the parameter higher-ranked, so
// the closure cannot name `'static` there (E0308-style mismatch).
fn apply<F: FnMut(&str)>(mut f: F) {
    let s = String::from("inner");
    f(&s);
}
fn main() {
    let mut keep: Vec<&'static str> = Vec::new();
    apply(|x: &'static str| keep.push(x));
    println!("{}", keep.len());
}
