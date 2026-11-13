// `*m` through a generic user Deref whose target is still a literal's open
// type (`ManuallyDrop::new(5)`): a later use fixes it, as in rustc.
use std::mem::ManuallyDrop;
fn main() {
    let m = ManuallyDrop::new(5);
    let x: u8 = *m;
    let n = ManuallyDrop::new(250);
    let y = *n + 10u8;
    let f = ManuallyDrop::new(1.5);
    let z: f32 = *f * 2.0;
    println!("{} {} {}", x, y, z);
}
