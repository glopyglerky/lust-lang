// expect: L0511
mod pa {
    pub mod inner {
        pub(in pa) fn f() -> i32 {
            1
        }
    }
    pub fn g() -> i32 {
        inner::f()
    }
}
fn main() {
    println!("{}", pa::g());
}
