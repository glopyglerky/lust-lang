// expect: L0506
mod a {
    pub fn b() {}
}
use a::{::b};
fn main() {}
