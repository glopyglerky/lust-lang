// expect: L0510
// rustc E0624: `secret` is private to `api`; the call is in the crate root.
mod api {
    pub trait T {}
    impl dyn T + '_ {
        fn secret(&self) -> u32 {
            7
        }
    }
}
impl api::T for u32 {}
fn main() {
    let n = 0u32;
    let d: &dyn api::T = &n;
    println!("{}", d.secret());
}
