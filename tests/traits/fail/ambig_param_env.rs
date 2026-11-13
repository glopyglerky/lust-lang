// rustc-error: E0283
// lust-expect: L0610
// ARCH 2.4 steps 1 and 6: two param-env predicates match the
// obligation `T: From<?X>` and unification does not pick one, so
// selection must defer and fail as ambiguous, never take the
// first match. From the round-2 probe ambig_err.rs. Rustc also
// reports E0790 for the Default call form; E0283 is the class
// this case pins.
fn f<T: From<u8> + From<u16>>() -> T {
    let v = Default::default();
    T::from(v)
}
fn main() {
    let _: u32 = f();
}
