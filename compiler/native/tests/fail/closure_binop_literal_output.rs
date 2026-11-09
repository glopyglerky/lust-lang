// expect: L0110
// rustc E0271: `x + 1` with `x` not known yet resolves through `Add`,
// so the literal falls back to i32 and the output is i32, not u64.
fn main() {
    let c = |x| x + 1;
    let r: u64 = c(4294967295);
    println!("{}", r);
}
