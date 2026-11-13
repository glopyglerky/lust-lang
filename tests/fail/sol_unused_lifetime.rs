// expect: L0110
// Sol review probe `unused_lifetime`; rustc: E0392 lifetime parameter `'a` is never used
struct S<'a> { x: i32 }
fn main() {}
