// expect: L0802
// rustc E0432: a glob does not copy a `macro_rules!` definition, so
// `self::k` after `use super::*;` names nothing.
mod m2 {
    macro_rules! k {
        () => {
            1u32
        };
    }
    pub mod c {
        use super::*;
        use self::k as j;
        pub fn f() -> u32 {
            j!()
        }
    }
}
fn main() {
    println!("{}", m2::c::f());
}
