// expect: L0511
mod pa {
    pub mod inner {
        pub(in crate::pb) fn f() -> i32 {
            1
        }
        pub fn g() -> i32 {
            2
        }
    }
}
mod pb {}
fn main() {
    println!("{}", pa::inner::g());
}
