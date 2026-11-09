// expect: L0311
// rustc E0505: the slice from `&&Vec` still borrows `v`.
fn main() {
    let v = vec![1, 2];
    let r = &v;
    let s: &[i32] = &r;
    drop(v);
    println!("{:?}", s);
}
