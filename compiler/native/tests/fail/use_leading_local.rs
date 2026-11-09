// expect: L0506
mod a {
    pub fn b() {}
}
use ::a;
fn main() {}
