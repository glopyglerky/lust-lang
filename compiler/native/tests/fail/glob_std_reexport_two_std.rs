// expect: L0507
// rustc E0659: two globs bring different std items named `Result`.
mod a {
    pub use std::fmt::Result;
}
mod b {
    pub use std::io::Result;
}
use a::*;
use b::*;
fn f() -> Result<()> {
    Ok(())
}
fn main() {
    println!("{:?}", f().is_ok());
}
