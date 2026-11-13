// core's conversion errors implement `std::error::Error` (the source
// trait, library/core/src/error.rs), so they meet an `E: Error` bound
// and convert through a user `From` at `?` (fx-tryfrom). `dyn Error`
// waits on the dyn.std work (tests/reject/std_error_dyn.rs).
use std::error::Error;
use std::fmt;

fn show<E: Error>(e: E) -> String {
    format!("{} | {:?}", e, e)
}

#[derive(Debug)]
enum AppErr {
    Range(std::num::TryFromIntError),
    Char(std::char::CharTryFromError),
}
impl From<std::num::TryFromIntError> for AppErr {
    fn from(e: std::num::TryFromIntError) -> Self {
        AppErr::Range(e)
    }
}
impl From<std::char::CharTryFromError> for AppErr {
    fn from(e: std::char::CharTryFromError) -> Self {
        AppErr::Char(e)
    }
}
impl fmt::Display for AppErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppErr::Range(e) => write!(f, "range: {}", e),
            AppErr::Char(e) => write!(f, "char: {}", e),
        }
    }
}
impl Error for AppErr {}

fn letter(x: i64) -> Result<char, AppErr> {
    let v: u32 = x.try_into()?;
    Ok(char::try_from(v)?)
}

fn main() {
    println!("{}", show(u8::try_from(300i32).unwrap_err()));
    println!("{}", show(u8::try_from(-1i32).unwrap_err()));
    println!("{}", show(char::try_from(0xD800u32).unwrap_err()));
    println!("{}", show(u8::try_from('Ā').unwrap_err()));
    for x in [65i64, -1, 0xD800] {
        match letter(x) {
            Ok(c) => println!("ok {}", c),
            Err(e) => println!("{} / {}", e.to_string(), show(e)),
        }
    }
}
