// expect: L0104
// rustc E0599: no impl of `W` defines `tag` for `W<u32>`.
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
    println!("{}", W::<u32>::tag());
}
