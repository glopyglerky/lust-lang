// ExitCode as main's result: a struct field with derives, Option, Debug,
// and a `{self, ..}` std module import.
use std::process::{self, ExitCode};
#[derive(Debug, Clone, Copy, PartialEq)]
struct S {
    c: ExitCode,
    n: u8,
}
fn code(s: &S) -> ExitCode {
    if s.n > 2 { s.c.clone() } else { ExitCode::FAILURE }
}
fn main() -> ExitCode {
    let s = S { c: ExitCode::from(42), n: 3 };
    let o: Option<ExitCode> = Some(code(&s));
    println!("{:?} {:?} {}", s, o, s == S { c: ExitCode::from(42), n: 3 });
    if s.n == 0 {
        process::exit(5);
    }
    o.unwrap()
}
