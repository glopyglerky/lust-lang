// rustc-error: E0308
// lust-expect: L0110
// ARCH 2.2: a user enum named Ordering shadows only the flat name;
// `std::cmp::Ordering` still names core's enum, so returning the user's
// variant where core's Ordering is expected is a type mismatch.
enum Ordering {
    Less,
}
fn f() -> std::cmp::Ordering {
    Ordering::Less
}
fn main() {
    let _ = f();
}
