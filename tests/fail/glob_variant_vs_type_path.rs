// expect: L0507
// rustc E0659 (ambiguous) / E0433: variants bind in the type
// namespace too, so a glob of the enum and a glob of a module
// with a type of the variant's name make the path head ambiguous.
enum E { B }
mod b {
    pub struct B;
    impl B {
        pub fn v() -> u8 {
            3
        }
    }
}
use E::*;
use b::*;
fn main() {
    println!("{}", B::v());
}
