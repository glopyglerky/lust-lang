// expect: L0106
mod a {
    pub fn b() {}
}
use a::b;
use a::b;
fn main() {}
