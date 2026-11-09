// expect: L0110
// The backing array of a slice const must have the slice's element type.
const A: &[u8] = &[true];
fn main() {
    println!("{:?}", A);
}
