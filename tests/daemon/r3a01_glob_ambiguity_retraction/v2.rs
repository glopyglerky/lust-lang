mod a {
    pub fn other() -> u32 {
        0
    }
}
mod b {
    pub struct X;
    impl X {
        pub fn v() -> u32 {
            2
        }
    }
}
use a::*;
use b::*;
fn main() {
    println!("{}", X::v() + other());
}
