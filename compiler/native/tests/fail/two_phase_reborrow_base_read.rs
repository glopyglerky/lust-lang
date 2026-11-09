// expect: L0303
// rustc E0502: the receiver reborrows `*r`, so `r`'s loan of `v` is live
// while the argument reads `v`. Only the new `&mut *r` is two-phase.
fn main() {
    let mut v = vec![1];
    let r = &mut v;
    r.push(v[0]);
    println!("{:?}", v);
}
