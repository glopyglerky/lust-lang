// expect: L0612
// rustc E0277: `String: Copy` does not hold. A bounded generic const fn
// call in a static proves its bounds after collect.
struct W<T>(Option<T>);
impl<T: Copy> W<T> {
    const fn mk() -> W<T> {
        W(None)
    }
}
static S: W<String> = W::mk();
fn main() {
    println!("{}", S.0.is_none());
}
