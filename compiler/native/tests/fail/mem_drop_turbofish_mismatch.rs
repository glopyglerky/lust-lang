// expect: L0110
// rustc E0308: `drop::<u16>` takes a `u16`, by direct call and as a value.
fn main() {
    std::mem::drop::<u16>(1u8);
    let f = std::mem::drop::<u16>;
    f(2u8);
}
