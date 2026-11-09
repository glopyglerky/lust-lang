mod a {
    pub struct X;
    impl X {
        pub fn v() -> u32 {
            1
        }
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
    println!("{}", X::v());
}
