// expect: L0104
// rustc E0599: a trait imported in an inner block is out of scope
// after that block ends.
mod m {
    pub trait T {
        fn n(&self) -> u32;
    }
}
struct S;
impl m::T for S {
    fn n(&self) -> u32 {
        9
    }
}
fn main() {
    {
        use m::T as _;
        println!("{}", S.n());
    }
    println!("{}", S.n());
}
