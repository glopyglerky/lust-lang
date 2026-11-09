// expect: L0306
// rustc E0597: `impl FnOnce() -> &'static str` ties the closure's result to `'static`.
fn name(f: impl FnOnce() -> &'static str) -> usize {
    f().len()
}

fn main() {
    let s = String::from("abc");
    println!("{}", name(|| &s));
}
