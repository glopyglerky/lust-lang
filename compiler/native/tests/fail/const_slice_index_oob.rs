// expect: L0630
const B: &[u8] = b"ab";
const N: u8 = B[5];
fn main() {
    println!("{N}");
}
