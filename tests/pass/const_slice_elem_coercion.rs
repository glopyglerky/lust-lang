// Slice consts: array elements are coercion sites, so nested slices,
// repeats, and options take the slice's element type.
const A: &[u8] = &[1, 2];
const R: &[u8] = &[7; 3];
static N: &[&[u8]] = &[&[1], &[2, 3]];
const S: &[&str] = &["a", "b"];
const O: &[Option<u8>] = &[None, Some(3)];
const B: [u8; 2] = [1, 2];
const P: &[u8] = &B;
const T: &[(&[u8], char)] = &[(&[1], 'a')];
const E: &[u16] = &[];
const T2: &(&[u8],) = &(&[4, 5],);
fn main() {
    println!("{A:?} {R:?} {N:?} {S:?} {O:?} {P:?} {T:?} {E:?} {:?}", T2.0);
}
