// ExitCode in generic code: Debug and PartialEq bounds hold, as in Rust.
use std::process::ExitCode;
fn show<T: std::fmt::Debug>(t: T) -> String {
    format!("{:?}", t)
}
fn main() -> ExitCode {
    let c = ExitCode::from(2);
    println!("{}", show(c));
    assert_eq!(c, ExitCode::from(2));
    c
}
