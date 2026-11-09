// VecDeque ring layout (W-STD-C phase 2): as_slices splits,
// make_contiguous, rotates, shrink_to and shrink_to_fit head
// positions, and swap_remove_front, all against a tracked ghost
// head over the logical buffer (library/alloc/src/collections.rs).
use std::collections::VecDeque;

fn show(tag: &str, d: &VecDeque<i32>) {
    let (a, b) = d.as_slices();
    println!("{tag} len={} cap={} split=({},{}) {:?} {:?}", d.len(), d.capacity(), a.len(), b.len(), a, b);
}

fn main() {
    let mut d = VecDeque::with_capacity(8);
    for i in 0..6 { d.push_back(i); }
    d.pop_front();
    d.pop_front();
    d.push_back(6);
    d.push_back(7);
    d.push_back(8);
    show("base", &d);
    let s = d.make_contiguous();
    s[0] += 100;
    show("contig", &d);
    d.rotate_left(2);
    show("rotl2", &d);
    d.rotate_right(3);
    show("rotr3", &d);
    d.rotate_left(7);
    show("rotl7", &d);
    d.swap_remove_front(2);
    show("srf2", &d);
    d.shrink_to(12);
    show("shrink12", &d);
    d.shrink_to_fit();
    show("fit", &d);
    let mut e: VecDeque<i32> = VecDeque::new();
    e.push_front(1);
    e.push_front(2);
    show("fronts", &e);
    e.rotate_right(1);
    show("rotr1", &e);
}
