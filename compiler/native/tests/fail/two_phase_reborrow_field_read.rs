// expect: L0303
// rustc E0503: `r.0` reborrows through `r`, whose loan of `t` is live
// while the argument copies `t.1`.
fn main() {
    let mut t = (vec![1], 2);
    let r = &mut t;
    r.0.push(t.1);
    println!("{:?}", t);
}
