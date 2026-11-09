// expect: L0304
// rustc E0499: each `FnOnce` closure returns a reborrow of the same
// `&mut w` capture, and both results are alive.
fn main() {
    let mut w = vec![1u8];
    let r = true.then(|| &mut w);
    let r2 = true.then(|| &mut w);
    r.unwrap().push(3);
    println!("{:?}", r2.is_some());
}
