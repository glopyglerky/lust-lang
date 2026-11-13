// expect: L0104
// rustc E0277: `usize: From<u32>` does not exist (only u8/u16/bool
// convert to `usize`, exactly as core defines it).
fn main() {
    let a: usize = 5u32.into();
    println!("{}", a);
}
