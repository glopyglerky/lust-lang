// Ordering::then_with: the closure runs only when the receiver is Equal
// (through FnOnce), else the receiver is the answer and the closure drops.
use std::cmp::Ordering;
fn main() {
    let tie = || { println!("tie"); 3u8.cmp(&1) };
    println!("{:?}", Ordering::Equal.then_with(tie));
    let skip = || { println!("never"); Ordering::Less };
    println!("{:?}", Ordering::Greater.then_with(skip));
    println!("{:?} {:?}", 1.cmp(&1).then_with(|| 2.cmp(&5)), 4.cmp(&1).then_with(|| 2.cmp(&5)));
}
