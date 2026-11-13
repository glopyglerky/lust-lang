// expect: L0630
// rustc E0658: range indexing is not const-stable (the `Index`
// trait), in a const item or a const fn. Slice patterns and
// `split_at` are the const slicing forms (fx-blob).
static B: &[u8] = b"abcdef";
const S: &[u8] = &B[2..5];
fn main() {
    println!("{}", S.len());
}
