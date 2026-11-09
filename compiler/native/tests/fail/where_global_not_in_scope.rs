// expect: L0104
// rustc E0599: a global where clause does not bring its trait into scope
// (the default method made lust once call the stub through it).
mod m {
    pub trait Tr {
        fn go(&self) -> i32;
        fn dflt(&self) -> i32 {
            100
        }
    }
    impl Tr for super::A {
        fn go(&self) -> i32 {
            1
        }
    }
}

struct A;

fn f() -> i32
where
    A: m::Tr,
{
    A.go()
}

fn main() {
    println!("{}", f());
}
