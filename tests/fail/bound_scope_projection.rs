// expect: L0104
// rustc E0599: a bound on a projection (`I::Item: m::Tr`) does not put the trait in scope.
mod m {
    pub trait Tr {
        fn f(&self) -> i32;
    }
    impl Tr for u8 {
        fn f(&self) -> i32 {
            1
        }
    }
}
fn go<I: Iterator>(x: I::Item) -> i32
where
    I::Item: m::Tr,
{
    x.f()
}
fn main() {
    println!("{}", go::<std::vec::IntoIter<u8>>(1));
}
