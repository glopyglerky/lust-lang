// Vec::spare_capacity_mut: the MaybeUninit view of len..capacity (ZST capacity usize::MAX), written through MaybeUninit::write.
use std::mem::MaybeUninit;
fn fill(buf: &mut [MaybeUninit<u8>]) -> usize {
    for (i, b) in buf.iter_mut().enumerate().take(3) {
        b.write(i as u8 + 1);
    }
    buf.len()
}
fn main() {
    let mut v: Vec<u8> = Vec::with_capacity(10);
    v.push(9);
    let s = v.spare_capacity_mut();
    println!("{} {:?}", s.len(), s[0]);
    println!("{}", fill(v.spare_capacity_mut()));
    let mut w: Vec<String> = Vec::new();
    println!("{}", w.spare_capacity_mut().len());
    w.reserve(5);
    let sp = w.spare_capacity_mut();
    let _ = &sp[1];
    println!("{} {}", sp.len(), w.len());
    let mut z: Vec<()> = vec![(), ()];
    println!("{}", z.spare_capacity_mut().len());
    let mut e: Vec<u64> = vec![1, 2, 3];
    println!("{}", e.spare_capacity_mut().len());
}
