// expect: L0612
// rustc E0277: no `TryFrom<f32>` for `u8` in core.
fn main() {
    println!("{:?}", u8::try_from(1.5f32));
}
