// expect: L0110
// rustc E0277: a tuple value whose last element is unsized is unsized.
fn main() {
    let v: (u8, str) = loop {};
    let _ = v;
}
