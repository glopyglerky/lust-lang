// expect: L0110
// ExitCode is From<u8> only.
use std::process::ExitCode;
fn main() -> ExitCode {
    let n: i32 = 3;
    ExitCode::from(n)
}
