// expect: L0001
// rustc: named argument never used.
fn main() {
    println!("{a}", a = 1, b = 2);
}
