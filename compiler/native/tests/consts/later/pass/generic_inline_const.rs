// stage: W-CHECK-G
// ARCH 2.13: an inline const block in a generic body. It is
// checked in generic form, then evaluated again per instance
// at mono. This is the ts-rust shape at core.rs:4085. It
// gates with W-CHECK-G and W-MONO in wave B.
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
    fn get(&self) -> &V {
        self.value.as_ref().unwrap()
    }
}
fn main() {
    let a = Slot::new(3u8);
    let b = Slot::new(4u64);
    println!("{} {}", a.get(), b.get());
}
