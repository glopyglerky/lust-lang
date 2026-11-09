// expect: L0612
// rustc E0277: the default `String` does not satisfy `T: Copy`.
struct S<T: Copy = String> {
    a: T,
}

fn main() {}
