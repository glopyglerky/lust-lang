// expect: L0112
// rustc E0277: `repeat_n` needs `T: Clone`.
struct S(u8);
fn main() {
    let n = std::iter::repeat_n(S(1), 2).count();
    println!("{n}");
}
