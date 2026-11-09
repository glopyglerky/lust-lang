// rustc-error: E0015
// lust-expect: L0631
// ARCH 2.13: const eligibility runs before any emission. A
// plain fn is not callable in a const initializer, even a
// local one with a trivial body.
fn make() -> u32 {
    42
}
const X: u32 = make();
fn main() {
    println!("{X}");
}
