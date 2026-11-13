// expect: L0110
// rustc E0271: with both operands open, `a + b` resolves through `Add`;
// the literals fall back to i32, so the output is i32, not u64.
fn main() {
    let add = |a, b| a + b;
    let r: u64 = add(4294967295, 1);
    println!("{}", r);
}
