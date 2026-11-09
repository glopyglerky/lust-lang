// `impl dyn T` methods are reachable where their visibility allows: the
// defining module and its descendants for a private one, the crate for
// `pub(crate)` and `pub(in crate)`, the parent for `pub(super)`.
mod api {
    pub trait T {
        fn n(&self) -> u32;
    }
    impl dyn T + '_ {
        fn private(&self) -> u32 {
            self.n() + 1
        }
        pub(crate) fn krate(&self) -> u32 {
            self.n() + 2
        }
        pub(super) fn parent(&self) -> u32 {
            self.n() + 3
        }
        pub(in crate) fn in_crate(&self) -> u32 {
            self.n() + 4
        }
    }
    pub mod child {
        pub fn call(d: &dyn super::T) -> u32 {
            d.private() * 10
        }
    }
    pub fn call(d: &dyn T) -> u32 {
        d.private()
    }
}
mod other {
    pub fn call<X>(d: &dyn crate::api::T, _x: X) -> u32 {
        d.krate() + d.in_crate()
    }
}
impl api::T for u32 {
    fn n(&self) -> u32 {
        *self
    }
}
fn main() {
    let n = 5u32;
    let d: &dyn api::T = &n;
    println!("{} {} {}", api::call(d), api::child::call(d), d.parent());
    println!("{} {}", d.krate(), other::call(d, 1u8));
}
