// expect: L0303
// rustc E0502: the bindings of `for (a, _b) in &v` borrow `v`, so `v.clear()` conflicts with `keep`.
fn main() {
    let mut v = vec![(1u8, 2u8)];
    let mut keep = Vec::new();
    for (a, _b) in &v {
        keep.push(a);
    }
    v.clear();
    println!("{:?}", keep);
}
