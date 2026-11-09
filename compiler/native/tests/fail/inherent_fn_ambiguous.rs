// expect: L0720
// rustc E0034: `W<_>` has two `tag` fns, one per impl.
struct W<T>(T);
impl W<u8> {
    fn tag() -> &'static str {
        "u8"
    }
}
impl W<u16> {
    fn tag() -> &'static str {
        "u16"
    }
}
fn main() {
    println!("{}", W::tag());
}
