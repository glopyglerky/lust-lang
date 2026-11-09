// expect: L0735
// rustc E0521: `apply` may pass a borrow of its own `s`, but the closure
// keeps its argument as `'a`.
fn apply<F: FnMut(&String)>(mut f: F) {
    let s = String::from("inner");
    f(&s);
}
fn outer<'a>(keep: &mut Vec<&'a String>) {
    let f = |x: &'a String| keep.push(x);
    apply(f);
}
fn main() {
    let mut v = Vec::new();
    outer(&mut v);
    println!("{:?}", v);
}
