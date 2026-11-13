// expect: L0115
// rustc E0004: BacktraceStatus is #[non_exhaustive], so a match needs `_`.
use std::backtrace::{Backtrace, BacktraceStatus};
fn main() { match Backtrace::disabled().status() { BacktraceStatus::Disabled => {}, BacktraceStatus::Captured => {}, BacktraceStatus::Unsupported => {} } }
