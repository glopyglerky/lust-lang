// expect: L0612
// ExitCode is not Hash.
use std::process::ExitCode;
fn need_hash<T: std::hash::Hash>(_t: T) {}
fn main() {
    need_hash(ExitCode::SUCCESS);
}
