// A const item operand is re-evaluated per element, so it needs no Copy.
use std::sync::atomic::{AtomicUsize, Ordering};
const A: AtomicUsize = AtomicUsize::new(0);
static Y: [AtomicUsize; 3] = [A; 3];
struct S(u8);
const B: S = S(7);
fn main() {
    Y[1].fetch_add(5, Ordering::SeqCst);
    let a = [B; 2];
    println!("{} {}", Y[0].load(Ordering::SeqCst) + Y[1].load(Ordering::SeqCst), a[1].0);
}
