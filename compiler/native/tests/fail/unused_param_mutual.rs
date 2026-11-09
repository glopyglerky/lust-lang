// expect: L0110
// rustc: type parameter `T` is only used recursively, through another type.
struct A<T>(Box<B<T>>);
struct B<T>(Option<Box<A<T>>>);

fn main() {}
