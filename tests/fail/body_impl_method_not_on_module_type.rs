// expect: L0104
// rustc E0599: the block type's method is not on the module type it shadows.
struct S;
fn f() -> u32 {
    struct S;
    impl S { fn y(&self) -> u32 { 5 } }
    S.y()
}
fn main() { println!("{} {}", f(), S.y()); }
