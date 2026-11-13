// expect: L0110
// rustc E0308: replacen's count is a usize.
fn main() {
    println!("{}", "ab".replacen(|c: char| c == 'a', "x", 1u8));
}
