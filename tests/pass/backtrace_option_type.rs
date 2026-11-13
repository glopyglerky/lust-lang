// `std::backtrace::Backtrace` by its imported name (was tests/reject:
// std::backtrace is shipped since tf-p6 R16).
use std::backtrace::Backtrace;

fn main() {
    let b: Option<Backtrace> = None;
    println!("{}", b.is_none());
}
