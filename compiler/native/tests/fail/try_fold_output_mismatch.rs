// expect: L0110
// rustc E0271: the closure's `Option<u64>` output must be the `u32` accumulator.
fn main() {
    let v = vec![1u32, 2];
    let r = v.iter().try_fold(0u32, |a, x| Some(a as u64 + *x as u64));
    println!("{:?}", r);
}
