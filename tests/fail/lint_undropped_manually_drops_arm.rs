// expect: L0543
// rustc undropped_manually_drops: a Copy ManuallyDrop still fires as a whole match arm body whose argument has side effects.
use std::mem::ManuallyDrop;
fn main() {
    match 1 {
        _ => drop(ManuallyDrop::new(5)),
    }
    println!("done");
}
