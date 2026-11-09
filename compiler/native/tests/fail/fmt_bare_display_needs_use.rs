// expect: L0102
// rustc E0405: `Display` is not in the prelude.
struct A;
impl Display for A {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a")
    }
}
fn main() {
    println!("{}", A);
}
