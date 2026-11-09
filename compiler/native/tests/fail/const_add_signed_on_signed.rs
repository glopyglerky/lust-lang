// expect: L0630
// rustc E0599: `wrapping_add_signed` exists only on unsigned integer types.
const A: i16 = 16384;
const SI: i16 = -1;
fn main() {
    println!("{:?}", const { A.wrapping_add_signed(SI) });
}
