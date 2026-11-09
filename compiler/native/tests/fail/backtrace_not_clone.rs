// expect: L0104
// rustc E0599: Backtrace is not Clone.
fn main() { let b = std::backtrace::Backtrace::disabled(); let _c = b.clone(); }
