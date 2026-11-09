use std::sync::atomic::Ordering;
fn f(o: Ordering) -> u8 { match o { Ordering::Relaxed => 0, Ordering::SeqCst => 4, _ => 9 } }
fn main() { println!("{} {} {}", f(Ordering::SeqCst), f(Ordering::Relaxed), f(Ordering::AcqRel)); }
