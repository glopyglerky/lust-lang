// ARCH 2.13, A1 form of the ts-rust shape at core.rs:472
// (`[const { OnceLock::new() }; CHUNKS]`). The real OnceLock
// form is later/pass/std_oncelock_repeat.rs and gates with
// W-THREADS. Array repeat evaluates the inline const once per
// element slot.
struct OnceLike<T> {
    value: Option<T>,
}
impl<T> OnceLike<T> {
    const fn new() -> Self {
        OnceLike { value: None }
    }
    fn is_empty(&self) -> bool {
        self.value.is_none()
    }
}
const CHUNKS: usize = 4;
static SLOTS: [OnceLike<u32>; CHUNKS] = [const { OnceLike::new() }; CHUNKS];
// A repeat whose element is not Copy and not a const block
// would not compile; the scalar repeat below is the plain form.
static COUNTS: [u8; 3] = [9; 3];
fn main() {
    let empty = SLOTS.iter().filter(|s| s.is_empty()).count();
    println!("{empty} {} {}", SLOTS.len(), COUNTS[1]);
}
