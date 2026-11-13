// expect: L0630
// rustc E0308: `W<u16>` is not `W<u8>`.
struct W<T>(Option<T>);
impl<T> W<T> {
    const fn mk() -> W<T> {
        W(None)
    }
}
static S: W<u8> = W::<u16>::mk();
fn main() {
    println!("{}", S.0.is_none());
}
