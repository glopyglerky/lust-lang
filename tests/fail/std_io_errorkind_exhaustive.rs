// expect: L0115
// ErrorKind is non_exhaustive: a match outside std needs a wildcard arm.
use std::io::ErrorKind;
fn f(k: ErrorKind) -> u8 {
    match k {
        ErrorKind::NotFound => 1,
        ErrorKind::Other => 2,
    }
}
fn main() {
    println!("{}", f(ErrorKind::Other));
}
