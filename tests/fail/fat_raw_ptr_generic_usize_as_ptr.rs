// expect: L0116
// rustc E0606: an integer does not cast to a pointer that may be wide (pre-existing false accept, tf-fatptr).
fn f<T: ?Sized>(p: usize) -> *const T {
    p as *const T
}
fn main() {}
