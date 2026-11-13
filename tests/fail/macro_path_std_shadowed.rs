// expect: L0802
// rustc E0433: the local module `std` shadows the std crate, and it has no `println`.
mod std {}
fn main() {
    std::println!("a");
}
