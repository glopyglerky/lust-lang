use std::cell::Cell;
fn main() {
    let mut v: Vec<(u32, Cell<u32>)> = (0..100).map(|i| ((i * 37) % 11, Cell::new(i))).collect();
    let mut calls = 0;
    v.sort_by(|a, b| { calls += 1; a.0.cmp(&b.0) });
    let ids: Vec<u32> = v.iter().map(|p| p.1.get()).collect();
    println!("{} {:?}", calls, &ids[..20]);
    let mut w: Vec<(u32, Cell<u32>)> = (0..100).map(|i| ((i * 37) % 11, Cell::new(i))).collect();
    let mut c2 = 0;
    w.sort_unstable_by(|a, b| { c2 += 1; a.0.cmp(&b.0) });
    let ids2: Vec<u32> = w.iter().map(|p| p.1.get()).collect();
    println!("{} {:?}", c2, &ids2[..20]);
}
