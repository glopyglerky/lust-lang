// expect: L0301
// rustc E0382: `r` moves into the first argument before `r.len()` runs.
fn f2(a: Vec<String>, b: usize) -> usize {
    a.len() + b
}
fn main() {
    let r = vec![String::new()];
    println!("{}", f2(r, r.len()));
}
