// expect: L0510
mod pa {
    mod inner {
        pub(super) fn f() -> i32 {
            1
        }
    }
    pub use self::inner::*;
}
fn main() {
    println!("{}", pa::f());
}
