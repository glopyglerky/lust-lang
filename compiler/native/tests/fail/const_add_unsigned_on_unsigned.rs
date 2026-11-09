// expect: L0630
// rustc E0599: `checked_add_unsigned` exists only on signed integer types.
const A: u32 = 7;
fn main() {
    println!("{:?}", const { A.checked_add_unsigned(1u32) });
}
