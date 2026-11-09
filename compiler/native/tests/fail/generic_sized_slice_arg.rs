// expect: L0612
// rustc E0277: `T` is implicitly `Sized`, and `[u8]` is not.
fn f<T>(_x: &T) -> u8 {
    1
}

fn main() {
    let a = [1u8, 2];
    let s: &[u8] = &a;
    println!("{}", f(s));
}
