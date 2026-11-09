// expect: L0110
// rustc: an `impl Trait` must list every type param in scope in its
// `use<..>` list.
fn wrap<T>(t: T) -> impl Sized + use<> {
    let _ = t;
    1
}
fn main() {
    let _ = wrap(1u8);
}
