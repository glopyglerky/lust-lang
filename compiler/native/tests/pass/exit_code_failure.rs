// main returns ExitCode::FAILURE after a handled error.
use std::process::ExitCode;
fn run() -> Result<u8, String> {
    Err(String::from("no"))
}
fn main() -> ExitCode {
    match run() {
        Ok(n) => ExitCode::from(n),
        Err(e) => {
            println!("error: {}", e);
            ExitCode::FAILURE
        }
    }
}
