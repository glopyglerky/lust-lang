// expect: L0001
mod a {
    pub fn b() {}
    pub fn c() {}
}
use a::{b,,c};
fn main() {}
