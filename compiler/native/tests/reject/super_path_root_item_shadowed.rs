// expect: L0505
// `super::f` names the crate-root `f`, but the checker would read the
// rewritten name as the module's own `f`; lust rejects it for now.
fn f() -> i32 { 1 }
mod m {
    fn f() -> i32 { 2 }
    pub fn p() -> i32 { f() * 10 + super::f() }
}
fn main() { println!("{}", m::p()); }
