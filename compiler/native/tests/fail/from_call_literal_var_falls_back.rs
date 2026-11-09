// expect: L0612
// rustc E0277: `c` falls back to i32, and `u32: From<i32>` does not hold.
fn main() {
    let c = 3;
    let d: u32 = From::from(c);
    println!("{}", d);
}
