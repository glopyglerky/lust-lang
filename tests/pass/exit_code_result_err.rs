// main returning Result<ExitCode, E>: Err prints `Error: ..` and exits 1.
use std::process::ExitCode;
fn main() -> Result<ExitCode, String> {
    println!("start");
    Err(String::from("bad"))
}
