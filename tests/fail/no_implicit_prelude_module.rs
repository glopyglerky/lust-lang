// expect: L0522
// rustc E0425: no `Option` in a #[no_implicit_prelude] module.
#[no_implicit_prelude]
mod m {
    pub fn f() -> Option<i32> { ::std::option::Option::None }
}
fn main() { println!("{:?}", m::f()); }
