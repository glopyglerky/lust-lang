// panic: assertion failed: n <= self.len()
// Rotating a VecDeque by more than its length is std's assert.
use std::collections::VecDeque;
fn main() {
    let mut d = VecDeque::from([1, 2, 3]);
    d.rotate_left(2);
    println!("{:?}", d);
    d.rotate_left(4);
    println!("unreachable");
}
