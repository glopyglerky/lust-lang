// expect: L0104
// rustc E0599: the block type's const is not on the module type it shadows.
struct S;
fn f() -> u32 {
    struct S;
    impl S { const Y: u32 = 5; }
    S::Y
}
fn main() { println!("{} {}", f(), S::Y); }
