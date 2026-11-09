// bool has its own `clamp` in core's cmp.rs: an assert, not the
// integer text with the bounds.
// panic: assertion failed: min <= max
fn main() {
    let n = std::env::args().count();
    println!("{}", (n > 0).clamp(false, true));
    println!("{}", (n > 0).clamp(true, false));
}
