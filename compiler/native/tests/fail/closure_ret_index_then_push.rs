// expect: L0303
// rustc E0502: `r` borrows `v` through the closure; `v.push` needs
// `&mut v` while `r` is live.
fn main() {
    let mut v = vec![1];
    let first = || &v[0];
    let r = first();
    v.push(2);
    println!("{}", r);
}
