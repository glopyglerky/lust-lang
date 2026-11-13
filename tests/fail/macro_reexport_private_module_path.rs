// expect: L0802
// rustc E0603: module `b` is private, so `a::b::k!` is not reachable.
mod a {
    mod b {
        macro_rules! k {
            () => {
                2u32
            };
        }
        pub(crate) use k;
    }
    pub fn g() -> u32 {
        b::k!()
    }
}
fn main() {
    println!("{} {}", a::g(), a::b::k!());
}
