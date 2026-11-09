// panic: base of integer logarithm must be at least 2
// ilog checks its base first, at the caller (core's #[track_caller]) (R13).
fn main() {
    let n = std::env::args().count();
    println!("{}", (n as u32 + 8).ilog(n as u32 + 1));
    println!("{}", 0u32.checked_ilog(5).is_none());
    println!("{}", (n as u32 - 1).ilog(n as u32));
}
