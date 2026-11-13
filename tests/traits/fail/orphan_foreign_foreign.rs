// rustc-error: E0117
// ARCH 2.3: the orphan rule needs the trait or a covering type
// to be local. Display and u8 are both foreign to this crate.
// Accepted sides: pass/orphan_local_sides_ok.rs.
impl std::fmt::Display for u8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("x")
    }
}
fn main() {}
