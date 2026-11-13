// expect: L0612
// rustc E0277: the default `[u8]` is not `Sized`, which `T` needs.
struct S<T = [u8]> {
    a: Box<T>,
}

fn main() {}
