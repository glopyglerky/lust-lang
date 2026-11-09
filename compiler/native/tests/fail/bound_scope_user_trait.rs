// expect: L0104
// rustc E0599: `T: m::Tr` gives `x.f()` for `x: T`, but `Tr` is not in scope for `5i32.f()`.
mod m {
    pub trait Tr {
        fn f(&self) -> i32;
    }
    impl Tr for i32 {
        fn f(&self) -> i32 {
            *self + 1
        }
    }
}
fn go<T: m::Tr>(x: T) -> i32 {
    x.f() + 5i32.f()
}
fn main() {
    println!("{}", go(1i32));
}
