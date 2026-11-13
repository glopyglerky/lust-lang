// expect: L0303
// rustc E0502: `r` (a `&mut` of the iterator over `v`) is used after the push.
fn main() {
    let mut v = vec![1u32, 2, 3];
    let mut it = v.iter();
    let r = &mut it;
    v.push(1);
    println!("{:?}", r.next());
}
