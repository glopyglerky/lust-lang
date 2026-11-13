// expect: L0505
// set_var is an unsafe fn in edition 2024 (and lust has no unsafe std surface).
fn main() {
    std::env::set_var("A", "b");
}
