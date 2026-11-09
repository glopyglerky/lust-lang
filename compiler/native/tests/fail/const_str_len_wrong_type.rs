// expect: L0110
// rustc E0308: `str::len` is a `usize`.
const N: u32 = "abc".len();
fn main() {
    println!("{}", N);
}
