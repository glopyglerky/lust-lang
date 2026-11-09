// expect: L0303
// rustc E0502: the `Cow` from `v[..].into()` borrows `v`.
use std::borrow::Cow;
fn main() {
    let mut v = vec![1u32, 2, 3];
    let x: Cow<[u32]> = v[..].into();
    v.push(4);
    println!("{:?}", x);
}
