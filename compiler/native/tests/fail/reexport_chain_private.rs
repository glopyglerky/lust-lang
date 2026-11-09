// expect: L0510
mod pa {
    mod inner {
        pub fn f() -> i32 {
            1
        }
    }
    mod mid {
        pub(crate) use super::inner::f;
    }
    pub use self::mid::f;
}
fn main() {
    println!("{}", pa::f());
}
