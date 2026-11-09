// expect: L0110
// rustc: type parameter `T` is only used recursively (E0392 class).
struct S<T>(Box<S<T>>);

fn main() {}
