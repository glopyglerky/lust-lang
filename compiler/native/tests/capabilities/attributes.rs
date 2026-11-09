// capability: attributes: derive
// capability: attributes: allow, warn, and expect; deny and forbid for `unsafe_code` and tool lints
// capability: attributes: cfg and cfg_attr
// capability: attributes: doc
// capability: attributes: inline, cold, must_use, deprecated, track_caller
// capability: attributes: repr
// capability: attributes: non_exhaustive
// capability: attributes: default (on an enum variant, for #[derive(Default)])
// capability: attributes: path, macro_use, macro_export
// capability: attributes: recursion_limit
// expect: ok

#![recursion_limit = "256"]
#![allow(dead_code)]

/// Docs.
#[derive(Debug, Default)]
#[repr(C)]
#[non_exhaustive]
pub struct S {
    a: u32,
}

#[derive(Debug, Default)]
enum E {
    #[default]
    A,
    B,
}

#[macro_use]
mod m {
    #[macro_export]
    macro_rules! one {
        () => {
            1
        };
    }
}

#[cfg(test)]
fn only_in_tests() -> u32 {
    1
}

#[cfg(not(test))]
fn only_in_tests() -> u32 {
    0
}

#[cfg_attr(not(test), inline)]
#[cold]
#[must_use]
#[deprecated]
fn f() -> u32 {
    2
}

#[track_caller]
#[warn(unused)]
#[deny(unsafe_code, clippy::all)]
#[forbid(unsafe_code)]
#[expect(unused_variables)]
fn g() -> u32 {
    let unused = 1;
    3
}

#[allow(deprecated)]
fn main() {
    println!("{:?} {:?} {} {} {} {}", S::default(), E::default(), f(), g(), one!(), only_in_tests());
}
