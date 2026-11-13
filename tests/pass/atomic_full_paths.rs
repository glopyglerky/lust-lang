fn main() { let a = std::sync::atomic::AtomicUsize::new(3); a.fetch_add(1, std::sync::atomic::Ordering::SeqCst); println!("{:?} {:?}", a, core::sync::atomic::Ordering::Acquire); }
