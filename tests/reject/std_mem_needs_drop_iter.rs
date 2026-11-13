// expect: L0641
// `needs_drop` of an iterator adapter type: lust's drop answer for those
// is not exact yet, so it rejects. rustc prints `true`.
fn main() {
    println!("{}", std::mem::needs_drop::<std::vec::IntoIter<u8>>());
}
