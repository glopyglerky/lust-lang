// expect: L0630
// rustc E0080: `A::MAX + 1` overflows `u8` (through the alias).
type A = u8;
const X: A = A::MAX + 1;
fn main() {
    println!("{}", X);
}
