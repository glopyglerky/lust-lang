// expect: L0001
mod a {
    pub fn b() {}
}
use a::b as ;
fn main() {}
