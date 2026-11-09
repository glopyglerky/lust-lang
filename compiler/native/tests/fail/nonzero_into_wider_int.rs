// expect: L0612
// rustc E0277: core has `From<NonZero<u32>> for u32`, not for `u64`.
use std::num::NonZeroU32;
fn main() {
    let n = NonZeroU32::new(3).unwrap();
    let w: u64 = u64::from(n);
    println!("{w}");
}
