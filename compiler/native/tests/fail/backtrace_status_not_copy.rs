// expect: L0301
// rustc E0382: BacktraceStatus is not Copy.
use std::backtrace::Backtrace;
fn main() { let s = Backtrace::disabled().status(); let _t = s; let _u = s; }
