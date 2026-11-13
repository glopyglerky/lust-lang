// expect: L0301
// rustc E0382: `r` moves into the tuple before `r.len()` runs.
fn g(r: Vec<String>) -> (Vec<String>, usize) {
    (r, r.len())
}
fn main() {
    println!("{:?}", g(vec![]));
}
