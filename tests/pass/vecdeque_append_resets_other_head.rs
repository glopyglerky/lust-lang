// VecDeque::append empties `other` as std does (`len = 0, head = 0`): later pushes into
// `other` start at the buffer's front, which as_slices shows (tf-build: corpus scan,
// w-std-c-p2 ring4; lust kept the old head, so the pushes wrapped).
use std::collections::VecDeque;

fn main() {
    let mut o: VecDeque<i32> = VecDeque::with_capacity(8);
    for i in 0..7 {
        o.push_back(i);
    }
    for _ in 0..2 {
        o.pop_front();
        o.push_back(90);
    }
    let mut d: VecDeque<i32> = VecDeque::with_capacity(4);
    d.append(&mut o);
    println!("{:?} {} {}", d, o.len(), o.capacity());
    for i in 0..7 {
        o.push_back(100 + i);
        let (a, b) = o.as_slices();
        println!("{} {} {}", o.capacity(), a.len(), b.len());
    }
}
