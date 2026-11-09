// expect: L0110
// rustc E0596: a `&mut [T]` cannot come through the shared inner
// reference of `&mut &Vec<T>`.
fn main() {
    let mut v = vec![1, 2];
    let r = &v;
    let s: &mut [i32] = &mut &*r;
    s[0] = 3;
}
