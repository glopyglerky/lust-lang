// expect: L0104
// rustc E0599: core's slice `SplitN` is not Clone (str's `SplitN` is).
fn main() {
    let v = vec![1u8, 2, 3];
    let a = v.splitn(2, |x| *x == 2);
    println!("{}", a.clone().count());
}
