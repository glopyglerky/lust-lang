// expect: L0110
// rustc E0277: a `[u16]` cannot be collected from `u8` items (Vec::leak gives the target).
fn main() { let _s: &'static mut [u16] = Vec::leak((0..3u8).collect()); }
