// expect: L0112
// rustc E0277: `u32` does not implement `Pointer`.
fn main() {
    let x = 5u32;
    println!("{:p}", x);
}
