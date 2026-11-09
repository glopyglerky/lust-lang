// expect: L0110
// rustc E0277: a struct literal field moves its value, even when the
// field may be unsized.
struct S<T: ?Sized> {
    f: T,
}
fn main() {
    let s = String::from("hello");
    let x = &S { f: s[1..] };
    println!("{}", x.f.len());
}
