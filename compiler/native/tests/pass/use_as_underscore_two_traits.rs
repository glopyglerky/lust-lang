mod a {
    pub trait T {
        fn f(&self) -> i64;
    }
}
mod b {
    pub trait U {
        fn g(&self) -> i64;
    }
}
use a::T as _;
use b::U as _;
struct S;
impl a::T for S {
    fn f(&self) -> i64 {
        1
    }
}
impl b::U for S {
    fn g(&self) -> i64 {
        2
    }
}
fn main() {
    let s = S;
    println!("{} {}", s.f(), s.g());
}
