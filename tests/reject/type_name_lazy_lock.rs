// expect: L0641
// `LazyLock` comes from shipped std source and lust has no verified
// `type_name` path for it yet: a reject, not a wrong string. (A
// closure's type_name rejects the same way, at build time.)
fn main() {
    println!("{}", std::any::type_name::<std::sync::LazyLock<u8>>());
}
