// expect: L0802
// rustc E0433: `println` is a std macro; core does not define it.
fn main() {
    core::println!("a");
}
