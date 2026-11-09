// expect: L0104
// rustc E0599: ManuallyDrop<T>: Clone needs T: Clone.
use std::mem::ManuallyDrop;
#[derive(Debug)]
struct NC;
fn main() {
    let m = ManuallyDrop::new(NC);
    let n = m.clone();
    println!("{:?}", n);
}
