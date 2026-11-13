use std::sync::atomic::*;
fn main() {
    let a = AtomicI8::new(-128); println!("{} {:?}", a.fetch_sub(1, Ordering::SeqCst), a);
    let b = AtomicU8::new(0); println!("{} {:?}", b.fetch_sub(1, Ordering::SeqCst), b);
    let c = AtomicI16::new(i16::MAX); println!("{} {:?}", c.fetch_add(1, Ordering::SeqCst), c);
    let d = AtomicU32::new(7); println!("{} {} {:?}", d.fetch_nand(3, Ordering::SeqCst), d.fetch_xor(0xff, Ordering::AcqRel), d);
    let e = AtomicI64::new(-1); println!("{} {} {:?}", e.fetch_max(-5, Ordering::SeqCst), e.fetch_min(i64::MIN, Ordering::Release), e);
    let f = AtomicU64::new(u64::MAX); println!("{} {:?}", f.fetch_max(3, Ordering::SeqCst), f.fetch_min(3, Ordering::SeqCst));
    let g = AtomicIsize::new(isize::MIN); println!("{:?} {:?}", g.swap(4, Ordering::SeqCst), g);
    let h = AtomicUsize::new(10); println!("{:?} {:?}", h.compare_exchange(9, 1, Ordering::SeqCst, Ordering::SeqCst), h.compare_exchange(10, 1, Ordering::AcqRel, Ordering::Acquire));
    let i = AtomicBool::new(true); println!("{} {} {} {:?}", i.fetch_nand(true, Ordering::SeqCst), i.fetch_or(false, Ordering::SeqCst), i.fetch_not(Ordering::SeqCst), i);
    println!("{:x?} {:#?} {:5?}|", AtomicU32::new(255), AtomicBool::new(false), AtomicI8::new(-3));
}
