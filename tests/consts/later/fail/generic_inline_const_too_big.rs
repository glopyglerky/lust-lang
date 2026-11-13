// stage: W-CHECK-G
// rustc-error: a link value over 32 bytes goes in a Box
// ARCH 2.13: an instance whose inline const assert is false
// fails at mono, even though the generic body checked clean.
// Pair with later/pass/generic_inline_const.rs.
use std::mem::size_of;
struct Slot<V> {
    value: Option<V>,
}
impl<V> Slot<V> {
    fn new(value: V) -> Self {
        const {
            assert!(size_of::<Option<V>>() <= 32, "a link value over 32 bytes goes in a Box");
        }
        Slot { value: Some(value) }
    }
}
fn main() {
    let big = Slot::new([0u8; 40]);
    println!("{}", big.value.unwrap()[0]);
}
