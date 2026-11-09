// Glob re-exports bind each name with the narrower of the `use`
// visibility and the item's own: names stay usable inside the region
// both reach.
mod pa {
    mod inner {
        pub(super) fn f() -> i32 {
            1
        }
        pub fn g() -> i32 {
            2
        }
        pub(crate) struct S {
            pub(crate) x: i32,
        }
        pub(super) enum E {
            A,
            B,
        }
    }
    pub use self::inner::E::*;
    pub use self::inner::*;
    pub fn uses() -> i32 {
        let n = [A, B].len() as i32;
        f() + self::f() + n
    }
    pub(crate) use self::inner::g as g2;
}
fn main() {
    let s = pa::S { x: 4 };
    println!("{} {} {} {}", pa::g(), pa::uses(), s.x, pa::g2());
}
