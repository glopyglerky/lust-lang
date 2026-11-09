// expect: L0110
// rustc E0308: the turbofish fixes the Ok type to u16.
fn main() { let r = Ok::<u16, u8>(1u8); println!("{:?}", r); }
