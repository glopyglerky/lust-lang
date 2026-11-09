// expect: L0630
// rustc E0080: `split_at(2)` is inside the two-byte `é`, so the const panics.
const P: (&str, &str) = "héllo".split_at(2);
fn main() {
    println!("{:?}", P);
}
