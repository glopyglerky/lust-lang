// expect: L0311
// rustc E0505: the element borrows `v` through the subslice.
fn main() {
    let v = vec![1u8, 2, 3, 4];
    let r = &v[1..][0];
    drop(v);
    println!("{:?}", r);
}
