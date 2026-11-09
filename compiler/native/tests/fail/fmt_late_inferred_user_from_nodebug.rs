// expect: L0612
// rustc E0277: the conditional `impl<T: Debug> Debug for W<T>` does not
// hold for `W<ND>`; `W::from` binds `T` only at body end.
use std::fmt;
struct W<T>(T);
impl<T: fmt::Debug> fmt::Debug for W<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("W")
    }
}
impl<T> From<T> for W<T> {
    fn from(t: T) -> Self {
        W(t)
    }
}
struct ND;
fn main() {
    let c = W::from(ND);
    println!("{:?}", c);
}
