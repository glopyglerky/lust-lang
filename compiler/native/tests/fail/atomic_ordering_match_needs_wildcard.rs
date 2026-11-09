// expect: L0115
use std::sync::atomic::Ordering;
fn f(o: Ordering) -> u8 { match o { Ordering::Relaxed => 0, Ordering::Release => 1, Ordering::Acquire => 2, Ordering::AcqRel => 3, Ordering::SeqCst => 4 } }
fn main() { println!("{}", f(Ordering::SeqCst)); }
