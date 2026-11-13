// expect: L0306
// rustc E0716: a cast of a local is not a constant, so it is not promoted.
fn main() {
    let x = 5;
    let r: &'static u8 = &(x as u8);
    println!("{}", r);
}
