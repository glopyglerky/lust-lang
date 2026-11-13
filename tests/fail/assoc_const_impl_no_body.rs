// expect: L0632
// rustc: associated constant in `impl` without body. A trait's
// required associated const is fine; an impl must give a value.
struct S;
impl S {
    const C: u8;
}
fn main() {}
