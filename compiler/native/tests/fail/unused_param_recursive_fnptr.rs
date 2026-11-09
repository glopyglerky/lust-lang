// expect: L0110
// rustc: type parameter `T` is only used recursively, here under a fn pointer.
struct S<T>(fn() -> Option<Box<S<T>>>);

fn main() {}
