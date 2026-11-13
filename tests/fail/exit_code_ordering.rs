// expect: L0112
// ExitCode has no ordering.
use std::process::ExitCode;
fn main() -> ExitCode {
    let c = ExitCode::SUCCESS;
    let d = c;
    println!("{:?}", c < d);
    d
}
