// expect: L0306
// rustc E0597: a `let` closure passed where the bound's output is `&'static str` must return a `'static` borrow.
fn run<F: FnOnce() -> &'static str>(f: F) -> usize {
    f().len()
}

fn main() {
    let s = String::from("ab");
    let f = || s.as_str();
    println!("{}", run(f));
}
