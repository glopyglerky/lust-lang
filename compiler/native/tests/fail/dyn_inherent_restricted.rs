// expect: L0510
// rustc E0624: `pub(in crate::api)` does not reach the sibling module `other`.
mod api {
    pub trait T {}
    impl dyn T + '_ {
        pub(in crate::api) fn secret(&self) -> u32 {
            7
        }
    }
}
mod other {
    pub fn call(d: &dyn crate::api::T) -> u32 {
        d.secret()
    }
}
impl api::T for u32 {}
fn main() {
    let n = 0u32;
    println!("{}", other::call(&n));
}
