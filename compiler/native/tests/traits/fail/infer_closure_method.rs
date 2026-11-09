// rustc-error: E0282
// lust-expect: L0610
// ARCH 2.9 and 2.12: a method call on a closure param needs a
// known head at that point. The later call does not rescue the
// body. From the Opus probe infer1.rs.
fn main() {
    let f = |x| x.len();
    println!("{}", f(vec![1u8]));
}
