// expect: L0301
// rustc E0382: `mem::forget` moves its argument.
struct S(u8);
fn main() {
    let s = S(1);
    std::mem::forget(s);
    println!("{}", s.0);
}
