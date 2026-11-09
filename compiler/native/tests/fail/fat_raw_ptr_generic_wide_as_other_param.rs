// expect: L0116
// rustc E0606: two `?Sized` params may have different metadata (pre-existing false accept, tf-fatptr).
fn f<T: ?Sized, U: ?Sized>(p: *const T) -> *const U {
    p as *const U
}
fn main() {}
