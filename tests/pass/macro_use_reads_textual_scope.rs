// `use k;` reads the macros textually in scope at the `use`: one an
// enclosing module defined before the `mod`, or a private self re-export
// that a child module then imports by path.
macro_rules! k {
    () => {
        1u32
    };
}
mod m2 {
    pub(crate) use k;
}
mod m3 {
    macro_rules! j {
        () => {
            2u32
        };
    }
    use j;
    pub mod c {
        use super::j;
        pub fn f() -> u32 {
            j!() + crate::m3::j!()
        }
    }
}
fn main() {
    println!("{} {}", m2::k!(), m3::c::f());
}
