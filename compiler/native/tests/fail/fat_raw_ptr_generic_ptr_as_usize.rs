// expect: L0116
// rustc E0606: a `*const T` with `T: ?Sized` may be wide, so it does not cast to an integer (pre-existing false accept, tf-fatptr).
fn f<T: ?Sized>(p: *const T) -> usize {
    p as usize
}
fn main() {}
