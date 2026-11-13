// expect: L0112
// rustc E0204: Copy needs every field Copy; the String field is not.
#[derive(Clone, Copy)]
struct S {
    id: std::num::NonZeroU32,
    name: String,
}
fn main() {
    let s = S { id: std::num::NonZeroU32::new(1).unwrap(), name: String::new() };
    let _t = s;
}
