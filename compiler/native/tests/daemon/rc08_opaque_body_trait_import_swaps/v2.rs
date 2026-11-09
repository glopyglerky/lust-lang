mod t {
    pub trait A {
        fn go(&self) -> i32;
    }
    pub trait B {
        fn go(&self) -> i32;
    }
    impl A for i32 {
        fn go(&self) -> i32 {
            *self + 1
        }
    }
    impl B for i32 {
        fn go(&self) -> i32 {
            *self * 10
        }
    }
}
use t::B;
fn mk() -> impl Copy {
    5i32
}
fn total() -> i32 {
    let _o = mk();
    let x: i32 = 4;
    x.go()
}
fn main() {
    println!("{}", total());
}
