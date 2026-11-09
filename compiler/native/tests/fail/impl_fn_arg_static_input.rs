// expect: L0306
// rustc E0597: an `impl FnMut(&'static str)` argument keeps its lifetime; the callee cannot pass a borrow of its local.
fn each(mut f: impl FnMut(&'static str, u8)) {
    let s = String::from("x");
    f(&s, 1);
}

fn main() {
    each(|s, _| println!("{}", s));
}
