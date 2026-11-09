// expect: L0110
// rustc E0392: lifetime parameter `'a` is never used (only recursively).
struct S<'a>(Option<Box<S<'a>>>);

fn main() {}
