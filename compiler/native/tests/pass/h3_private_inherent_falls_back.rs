// A private inherent method is no candidate outside its module: rustc's
// probe goes on to the visible trait method of the same name (also through
// autoderef and by-value forms); inside the module the inherent one wins.
mod m {
    pub struct S;
    impl S {
        fn h(&self) -> i32 { 1 }
    }
    pub trait Tr {
        fn h(&self) -> i32 { 2 }
    }
    impl Tr for S {}
    pub fn call() -> i32 { S.h() }
}
use m::Tr;
fn main() {
    let b = Box::new(m::S);
    println!("{} {} {} {}", m::call(), m::S.h(), b.h(), (&&m::S).h());
}
