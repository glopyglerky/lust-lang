// panic: panic_path_qualified_macro_location.rs:6:5:
// A path-qualified panic macro reports the column of the path's start.
fn main() {
    core::assert!(bb(1) == 1);
    std::assert_ne!(bb(1), 2);
    ::std::assert_eq!(bb(1), 2);
}
#[inline(never)]
fn bb<T>(x: T) -> T {
    x
}
