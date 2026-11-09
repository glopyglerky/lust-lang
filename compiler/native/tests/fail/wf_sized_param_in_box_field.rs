// expect: L0612
// rustc E0277: `W<str>` needs `str: Sized` in a field type too.
struct W<T>(T);

struct H {
    x: Box<W<str>>,
}

fn main() {}
