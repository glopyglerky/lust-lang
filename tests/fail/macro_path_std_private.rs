// expect: L0802
// rustc E0603: std::macros is private.
fn main() {
    std::macros::println!("a");
}
