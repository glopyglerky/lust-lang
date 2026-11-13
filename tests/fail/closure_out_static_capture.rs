// expect: L0306
// rustc E0597: the bound gives the closure the return type `&'static str`, so a borrow of a captured local cannot be its result.
fn run<F: FnMut() -> &'static str>(mut f: F) -> usize {
    f().len()
}

fn main() {
    let s = String::from("ab");
    println!("{}", run(|| s.as_str()));
}
