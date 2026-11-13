// VecDeque::drain leaves the ring head where std's bulk shift
// puts it (W-STD-C phase 2 ghost head).
use std::collections::VecDeque;
fn show(tag: &str, d: &VecDeque<i32>) {
    let (a, b) = d.as_slices();
    println!("{tag} len={} cap={} split=({},{}) {a:?}{b:?}", d.len(), d.capacity(), a.len(), b.len());
}
fn main() {
    let mut d = VecDeque::with_capacity(8);
    for i in 0..6 { d.push_back(i); }
    d.pop_front();
    d.pop_front();
    d.push_back(6);
    d.push_back(7);
    d.push_back(8);
    let drained: Vec<i32> = d.drain(1..4).collect();
    println!("drained {drained:?}");
    show("after", &d);
    {
        let mut dr = d.drain(1..3);
        println!("first {:?}", dr.next());
    }
    show("partial", &d);
}
