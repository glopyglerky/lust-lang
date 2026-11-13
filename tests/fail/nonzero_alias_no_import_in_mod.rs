// expect: L0102
// rustc E0433: the root's import does not reach the child module.
use std::num::NonZeroU8;
mod m {
    pub fn f() -> Option<u8> {
        NonZeroU8::new(1).map(|x| x.get())
    }
}
fn main() {
    let _ = NonZeroU8::MIN;
    println!("{:?}", m::f());
}
