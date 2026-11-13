// expect: L0506
// rustc E0432: a `macro_rules!` definition is not in its module's namespace,
// so `super::k` names nothing without a `use k;` in the parent.
mod m2 {
    macro_rules! k {
        () => {
            1u32
        };
    }
    pub mod c {
        use super::k;
        pub fn f() -> u32 {
            k!()
        }
    }
}
fn main() {
    println!("{}", m2::c::f());
}
