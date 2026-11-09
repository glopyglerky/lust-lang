// expect: L0110
// rustc E0046: an impl of a std trait must define its required methods.
struct S(u8);
impl Clone for S {}
fn main() {
    println!("{}", S(1).0);
}
