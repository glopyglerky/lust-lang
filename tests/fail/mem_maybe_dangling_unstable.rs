// expect: L0505
// rustc E0658: core::mem::MaybeDangling is unstable (private in lust).
fn main() {
    let d = std::mem::MaybeDangling::new(1);
    let _ = d;
}
