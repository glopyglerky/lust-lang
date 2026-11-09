// expect: L0630
// rustc E0599: `saturating_neg` exists only on signed integer types.
const A: u8 = 7;
fn main() {
    println!("{:?}", const { A.saturating_neg() });
}
