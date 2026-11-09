use std::sync::atomic::*;
fn main() { println!("{} {} {}", std::mem::size_of::<AtomicBool>(), std::mem::size_of::<Option<AtomicBool>>(), std::mem::align_of::<AtomicU64>()); }
