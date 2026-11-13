// expect: L0612
// rustc E0277: the default `Vec<[u8]>` is not well-formed.
struct S<T = Vec<[u8]>> {
    a: Box<T>,
}

fn main() {}
