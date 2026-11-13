fn main() { use std::sync::atomic::*; let a = AtomicU32::new(3); fence(Ordering::SeqCst); println!("{:?}", a.swap(9, Ordering::Relaxed)); }
