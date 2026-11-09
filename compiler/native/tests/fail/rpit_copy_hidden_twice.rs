// expect: L0301
// rustc E0382: `impl Sized` is not `Copy` by its bounds, so the first
// `let` moves the value even though its hidden type `usize` is `Copy`.
fn len_of(v: &Vec<u32>) -> impl Sized {
    v.len()
}
fn main() {
    let v = vec![1, 2];
    let it = len_of(&v);
    let a = it;
    let b = it;
}
