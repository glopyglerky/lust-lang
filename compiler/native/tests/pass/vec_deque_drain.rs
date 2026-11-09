// VecDeque::drain over the source overlay's logical-order buffer:
// range forms, both ends, early drop, and a leaked Drain.

use std::collections::VecDeque;

struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

fn main() {
    let mut d: VecDeque<i32> = VecDeque::from(vec![1, 2, 3, 4, 5]);
    let mut it = d.drain(1..4);
    println!("{} {:?} {:?}", it.len(), it.next(), it.next_back());
    drop(it);
    println!("{d:?}");
    let c: Vec<i32> = d.drain(..).collect();
    println!("all {c:?} {d:?}");
    let mut d: VecDeque<i32> = VecDeque::from(vec![1, 2, 3, 4]);
    let r = 1..=2;
    println!("{:?} {:?}", d.drain(r).collect::<Vec<i32>>(), d);
    let mut d: VecDeque<D> = VecDeque::from(vec![D(0), D(1), D(2), D(3), D(4)]);
    {
        let mut it = d.drain(1..4);
        match it.next() {
            Some(x) => println!("take {}", x.0),
            None => println!("take none"),
        }
    }
    println!("left {}", d.len());
    let it = d.drain(..);
    std::mem::forget(it);
    println!("forget len={}", d.len());
    drop(d);
}
