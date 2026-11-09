// expect: L0612
// rustc E0277: `W<str>` needs `str: Sized`, even behind a reference.
struct W<T>(T);

fn f(_x: &W<str>) {}

fn main() {}
