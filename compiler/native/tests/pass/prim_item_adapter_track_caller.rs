// panic: argument of integer logarithm must be positive
// A #[track_caller] primitive method item called by an iterator adapter panics at core's
// FnMut::call_mut shim (core/src/ops/function.rs), as rustc reports it.
fn main() {
    let n = std::env::args().count();
    let v: Vec<u32> = [8u32, 100].iter().copied().map(u32::ilog2).collect();
    println!("{v:?}");
    let w: u32 = [n as u32 - 1].into_iter().map(u32::ilog10).sum();
    println!("{w}");
}
