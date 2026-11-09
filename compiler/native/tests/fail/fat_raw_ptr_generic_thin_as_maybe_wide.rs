// expect: L0116
// rustc E0607: a thin pointer does not cast to a pointer that may be wide (pre-existing false accept, tf-fatptr).
fn f<T: ?Sized>(p: *const u8) -> *const T {
    p as *const T
}
fn main() {}
