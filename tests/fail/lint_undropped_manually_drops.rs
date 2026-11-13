// expect: L0543
// rustc undropped_manually_drops (deny): drop of a ManuallyDrop<String> does nothing.
use std::mem::ManuallyDrop;
fn main() {
    let m = ManuallyDrop::new(String::from("x"));
    drop(m);
    println!("done");
}
