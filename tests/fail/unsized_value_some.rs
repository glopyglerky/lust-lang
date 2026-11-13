// expect: L0110
// rustc E0277: a variant ctor argument moves its value; `[u32]` has no
// known size.
fn main() {
    let v = vec![1u32, 2, 3];
    let x = Some(v[1..]);
    println!("{}", x.is_some());
}
