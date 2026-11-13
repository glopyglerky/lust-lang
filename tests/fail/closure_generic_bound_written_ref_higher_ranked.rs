// expect: L0733
// rustc E0521: `F: FnOnce(&String)` writes the reference: the argument
// is higher-ranked, so storing it escapes.
fn with<F: FnOnce(&String) -> usize>(f: F) -> usize { let s = String::from("in"); f(&s) }
fn main() {
    let mut keep: Vec<&String> = Vec::new();
    let n = with(|s| { keep.push(s); 0 });
    println!("{} {}", n, keep.len());
}
