// expect: L0110
// rustc E0308: `u32::isqrt` returns a `u32`, not a `u64`.
fn main() {
    let _f: fn(u32) -> u64 = u32::isqrt;
}
