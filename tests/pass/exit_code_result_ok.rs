// main returning Result<ExitCode, E>: Ok carries the exit status.
use std::process::ExitCode;
fn main() -> Result<ExitCode, String> {
    Ok(ExitCode::from(4))
}
