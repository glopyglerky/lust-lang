// expect: L0110
// rustc E0735: generic parameters cannot use `Self` in their defaults.
struct S<T = Self> {
    a: Option<Box<T>>,
}

fn main() {}
