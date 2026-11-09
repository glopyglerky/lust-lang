// expect: L0110
// rustc E0277: `forget` needs a sized value.
fn main() {
    let v = vec![1u8, 2];
    let s: &[u8] = &v;
    std::mem::forget(*s);
}
