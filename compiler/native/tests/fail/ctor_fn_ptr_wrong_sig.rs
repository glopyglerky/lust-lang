// expect: L0110
// rustc E0308: the constructor takes an `i32`, not a `u8`.
struct J(i32);
fn main() {
    let _f: fn(u8) -> J = J;
}
