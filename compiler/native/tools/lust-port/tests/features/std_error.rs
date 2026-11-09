// feature: std.error
// form: a user std::error::Error impl used through a bound, as in ts-rust diagnostics
use std::error::Error;
use std::fmt;
#[derive(Debug)]
struct Boom(i32);
impl fmt::Display for Boom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "boom {}", self.0) }
}
impl Error for Boom {}
fn show<E: Error>(e: &E) -> String { format!("{e}") }
fn main() {
    println!("{} {:?}", show(&Boom(3)), Boom(4));
}
