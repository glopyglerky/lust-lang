// expect: L0630
// Valid Rust. lust's const evaluator copies on a borrow, so it takes a `&mut`
// only as a const fn call argument (written back after the call); a `&mut`
// kept in a binding would lose its writes, so it is outside the evaluator.
const C: u32 = {
    let mut a = 1u32;
    let r = &mut a;
    *r += 1;
    a
};
fn main() {
    println!("{}", C);
}
