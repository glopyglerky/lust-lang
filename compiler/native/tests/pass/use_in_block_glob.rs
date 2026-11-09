// A glob `use` in a fn body brings the module's trait into scope
// for method calls, and its type names in with it.
mod m {
    pub trait T {
        fn n(&self) -> u32;
    }
    pub struct S(pub u32);
}
struct S;
impl m::T for S {
    fn n(&self) -> u32 {
        9
    }
}
fn main() {
    use m::*;
    let s = m::S(1);
    println!("{} {}", crate::S.n(), s.0);
}
