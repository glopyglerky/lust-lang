// expect: L0110
// rustc E0308: a width taken from an argument must be a `usize`.
fn main() {
    let w: u32 = 3;
    println!("[{:w$}]", 1);
}
