// ARCH 2.4 step 1, pass side of the two-predicate case: two
// param-env predicates match by trait, but a concrete argument
// fixes the input, so one predicate is picked. Contrast with
// fail/ambig_param_env.rs, where the input stays a variable.
fn from_u8<T: From<u8> + From<u16>>() -> T {
    T::from(3u8)
}
fn from_u16<T: From<u8> + From<u16>>() -> T {
    T::from(4u16)
}
fn main() {
    let a: u32 = from_u8();
    let b: u32 = from_u16();
    println!("{a} {b}");
}
