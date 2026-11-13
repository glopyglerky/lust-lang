// expect: L0720
// rustc E0034: two block-imported traits give the same method.
mod a {
    pub trait T {
        fn n(&self) -> u32;
    }
}
mod b {
    pub trait T {
        fn n(&self) -> u32;
    }
}
struct S;
impl a::T for S {
    fn n(&self) -> u32 {
        1
    }
}
impl b::T for S {
    fn n(&self) -> u32 {
        2
    }
}
fn main() {
    use a::T as _;
    use b::T as _;
    println!("{}", S.n());
}
