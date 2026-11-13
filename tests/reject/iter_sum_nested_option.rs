// expect: L0640
// Valid Rust: Option<Option<u8>>: Sum<Option<Option<u8>>> (u8: Sum<u8>
// two levels down); lust's expansion sums one level.
fn main() {
    let v = vec![Some(Some(1u8))];
    let r: Option<Option<u8>> = v.into_iter().sum();
    println!("{:?}", r);
}
