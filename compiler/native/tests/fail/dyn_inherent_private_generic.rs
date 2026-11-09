// expect: L0510
// rustc E0624: a generic body in another module calls the private method.
mod api {
    pub trait T {}
    impl dyn T + '_ {
        fn secret(&self) -> u32 {
            7
        }
    }
}
mod consumer {
    pub fn call<X>(d: &dyn crate::api::T, _x: X) -> u32 {
        d.secret()
    }
}
impl api::T for u32 {}
fn main() {
    let n = 0u32;
    println!("{}", consumer::call(&n, 0u8));
}
