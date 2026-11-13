// expect: L0612
// rustc E0277: `-x` needs `u64: Neg`; the integer variable becomes u64 later.
fn main() {
    let x = -5;
    let y: u64 = x;
    println!("{}", y);
}
